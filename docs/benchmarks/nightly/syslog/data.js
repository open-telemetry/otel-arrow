window.BENCHMARK_DATA = {
  "lastUpdate": 1765535685292,
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
          "id": "fb291bfcb973afe17b8b98827bb12ebcb1d59203",
          "message": "Retry processor (stateless) Ack/Nack (#1295)\n\nReplaces stateful retry processor with new stateless Ack/Nack support.\n\nThe component has been restructured with exactly same configuration used\nby the OTel Collector retry configuration, with 3 duration fields (in\nseconds) instead of a retry limit.\n\nNew tests written. Metrics updated.\n\nNote: stores the num_items calculated from the initial data in a 3-word\ncalldata, as a temporary measure. In the 10/21/2025 SIG meeting we\ndiscussed placing num_items in another field of the context,\npotentially, that could be a separate change.\n\n---------\n\nCo-authored-by: Utkarsh Umesan Pillai <66651184+utpilla@users.noreply.github.com>\nCo-authored-by: Laurent Quérel <laurent.querel@gmail.com>",
          "timestamp": "2025-10-24T15:24:14Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/fb291bfcb973afe17b8b98827bb12ebcb1d59203"
        },
        "date": 1761473591842,
        "tool": "customSmallerIsBetter",
        "benches": [
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
            "value": 16.712574790232686,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 18.63060775338627,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 25.1640625,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 25.296875,
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
            "value": 877540.6374024972,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13030978.581230007,
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
            "value": 20.00814518966167,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 20.957908807808508,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 21.22734375,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 21.34765625,
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
            "value": 335288.9941031771,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 12990532.142152522,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Network Utilization"
          },
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
            "value": 15.172114826344101,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 16.501119309600863,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 25.310546875,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 25.71875,
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
            "value": 293495.5447491827,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13098217.141814161,
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
            "value": 13.210462639089165,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 13.422032221791994,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.075,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.22265625,
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
            "value": 539214.856634961,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13122423.716008354,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - Network Utilization"
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
          "id": "fb291bfcb973afe17b8b98827bb12ebcb1d59203",
          "message": "Retry processor (stateless) Ack/Nack (#1295)\n\nReplaces stateful retry processor with new stateless Ack/Nack support.\n\nThe component has been restructured with exactly same configuration used\nby the OTel Collector retry configuration, with 3 duration fields (in\nseconds) instead of a retry limit.\n\nNew tests written. Metrics updated.\n\nNote: stores the num_items calculated from the initial data in a 3-word\ncalldata, as a temporary measure. In the 10/21/2025 SIG meeting we\ndiscussed placing num_items in another field of the context,\npotentially, that could be a separate change.\n\n---------\n\nCo-authored-by: Utkarsh Umesan Pillai <66651184+utpilla@users.noreply.github.com>\nCo-authored-by: Laurent Quérel <laurent.querel@gmail.com>",
          "timestamp": "2025-10-24T15:24:14Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/fb291bfcb973afe17b8b98827bb12ebcb1d59203"
        },
        "date": 1761560072794,
        "tool": "customSmallerIsBetter",
        "benches": [
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
            "value": 12.844974103381837,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 13.015380633175944,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.25546875,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.33203125,
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
            "value": 535359.3956894501,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13120016.610477943,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - Network Utilization"
          },
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
            "value": 15.922331558260655,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 16.184834061000156,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 25.226171875,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 25.38671875,
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
            "value": 295518.52093009296,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13163676.352549573,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - Network Utilization"
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
            "value": 23.017121665461126,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 28.037382227403434,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 25.024609375,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 25.26171875,
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
            "value": 335669.02713790565,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 12987331.089934837,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Network Utilization"
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
            "value": 16.848183783835776,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 17.56826332843327,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 21.228125,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 21.3125,
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
            "value": 877303.9839725798,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13030458.43114168,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - Network Utilization"
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
          "id": "f71769d880ddce56b4525bc88d709d9a7edec65a",
          "message": "chore(deps): update github workflow dependencies (#1349)\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n|---|---|---|---|\n|\n[github/codeql-action](https://redirect.github.com/github/codeql-action)\n| action | minor | `v4.30.9` -> `v4.31.0` |\n|\n[taiki-e/install-action](https://redirect.github.com/taiki-e/install-action)\n| action | patch | `v2.62.33` -> `v2.62.40` |\n\n---\n\n### Release Notes\n\n<details>\n<summary>github/codeql-action (github/codeql-action)</summary>\n\n###\n[`v4.31.0`](https://redirect.github.com/github/codeql-action/compare/v4.30.9...v4.31.0)\n\n[Compare\nSource](https://redirect.github.com/github/codeql-action/compare/v4.30.9...v4.31.0)\n\n</details>\n\n<details>\n<summary>taiki-e/install-action (taiki-e/install-action)</summary>\n\n###\n[`v2.62.40`](https://redirect.github.com/taiki-e/install-action/blob/HEAD/CHANGELOG.md#100---2021-12-30)\n\n[Compare\nSource](https://redirect.github.com/taiki-e/install-action/compare/v2.62.39...v2.62.40)\n\nInitial release\n\n[Unreleased]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.40...HEAD\n\n[2.62.40]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.39...v2.62.40\n\n[2.62.39]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.38...v2.62.39\n\n[2.62.38]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.37...v2.62.38\n\n[2.62.37]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.36...v2.62.37\n\n[2.62.36]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.35...v2.62.36\n\n[2.62.35]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.34...v2.62.35\n\n[2.62.34]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.33...v2.62.34\n\n[2.62.33]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.32...v2.62.33\n\n[2.62.32]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.31...v2.62.32\n\n[2.62.31]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.30...v2.62.31\n\n[2.62.30]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.29...v2.62.30\n\n[2.62.29]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.28...v2.62.29\n\n[2.62.28]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.27...v2.62.28\n\n[2.62.27]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.26...v2.62.27\n\n[2.62.26]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.25...v2.62.26\n\n[2.62.25]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.24...v2.62.25\n\n[2.62.24]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.23...v2.62.24\n\n[2.62.23]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.22...v2.62.23\n\n[2.62.22]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.21...v2.62.22\n\n[2.62.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.20...v2.62.21\n\n[2.62.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.19...v2.62.20\n\n[2.62.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.18...v2.62.19\n\n[2.62.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.17...v2.62.18\n\n[2.62.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.16...v2.62.17\n\n[2.62.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.15...v2.62.16\n\n[2.62.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.14...v2.62.15\n\n[2.62.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.13...v2.62.14\n\n[2.62.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.12...v2.62.13\n\n[2.62.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.11...v2.62.12\n\n[2.62.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.10...v2.62.11\n\n[2.62.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.9...v2.62.10\n\n[2.62.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.8...v2.62.9\n\n[2.62.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.7...v2.62.8\n\n[2.62.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.6...v2.62.7\n\n[2.62.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.5...v2.62.6\n\n[2.62.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.4...v2.62.5\n\n[2.62.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.3...v2.62.4\n\n[2.62.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.2...v2.62.3\n\n[2.62.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.1...v2.62.2\n\n[2.62.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.0...v2.62.1\n\n[2.62.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.13...v2.62.0\n\n[2.61.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.12...v2.61.13\n\n[2.61.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.11...v2.61.12\n\n[2.61.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.10...v2.61.11\n\n[2.61.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.9...v2.61.10\n\n[2.61.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.8...v2.61.9\n\n[2.61.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.7...v2.61.8\n\n[2.61.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.6...v2.61.7\n\n[2.61.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.5...v2.61.6\n\n[2.61.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.4...v2.61.5\n\n[2.61.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.3...v2.61.4\n\n[2.61.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.2...v2.61.3\n\n[2.61.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.1...v2.61.2\n\n[2.61.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.0...v2.61.1\n\n[2.61.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.60.0...v2.61.0\n\n[2.60.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.59.1...v2.60.0\n\n[2.59.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.59.0...v2.59.1\n\n[2.59.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.33...v2.59.0\n\n[2.58.33]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.32...v2.58.33\n\n[2.58.32]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.31...v2.58.32\n\n[2.58.31]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.30...v2.58.31\n\n[2.58.30]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.29...v2.58.30\n\n[2.58.29]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.28...v2.58.29\n\n[2.58.28]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.27...v2.58.28\n\n[2.58.27]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.26...v2.58.27\n\n[2.58.26]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.25...v2.58.26\n\n[2.58.25]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.24...v2.58.25\n\n[2.58.24]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.23...v2.58.24\n\n[2.58.23]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.22...v2.58.23\n\n[2.58.22]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.21...v2.58.22\n\n[2.58.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.20...v2.58.21\n\n[2.58.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.19...v2.58.20\n\n[2.58.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.18...v2.58.19\n\n[2.58.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.17...v2.58.18\n\n[2.58.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.16...v2.58.17\n\n[2.58.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.15...v2.58.16\n\n[2.58.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.14...v2.58.15\n\n[2.58.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.13...v2.58.14\n\n[2.58.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.12...v2.58.13\n\n[2.58.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.11...v2.58.12\n\n[2.58.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.10...v2.58.11\n\n[2.58.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.9...v2.58.10\n\n[2.58.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.8...v2.58.9\n\n[2.58.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.7...v2.58.8\n\n[2.58.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.6...v2.58.7\n\n[2.58.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.5...v2.58.6\n\n[2.58.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.4...v2.58.5\n\n[2.58.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.3...v2.58.4\n\n[2.58.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.2...v2.58.3\n\n[2.58.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.1...v2.58.2\n\n[2.58.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.0...v2.58.1\n\n[2.58.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.8...v2.58.0\n\n[2.57.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.7...v2.57.8\n\n[2.57.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.6...v2.57.7\n\n[2.57.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.5...v2.57.6\n\n[2.57.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.4...v2.57.5\n\n[2.57.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.3...v2.57.4\n\n[2.57.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.2...v2.57.3\n\n[2.57.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.1...v2.57.2\n\n[2.57.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.0...v2.57.1\n\n[2.57.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.24...v2.57.0\n\n[2.56.24]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.23...v2.56.24\n\n[2.56.23]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.22...v2.56.23\n\n[2.56.22]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.21...v2.56.22\n\n[2.56.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.20...v2.56.21\n\n[2.56.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.19...v2.56.20\n\n[2.56.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.18...v2.56.19\n\n[2.56.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.17...v2.56.18\n\n[2.56.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.16...v2.56.17\n\n[2.56.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.15...v2.56.16\n\n[2.56.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.14...v2.56.15\n\n[2.56.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.13...v2.56.14\n\n[2.56.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.12...v2.56.13\n\n[2.56.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.11...v2.56.12\n\n[2.56.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.10...v2.56.11\n\n[2.56.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.9...v2.56.10\n\n[2.56.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.8...v2.56.9\n\n[2.56.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.7...v2.56.8\n\n[2.56.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.6...v2.56.7\n\n[2.56.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.5...v2.56.6\n\n[2.56.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.4...v2.56.5\n\n[2.56.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.3...v2.56.4\n\n[2.56.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.2...v2.56.3\n\n[2.56.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.1...v2.56.2\n\n[2.56.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.0...v2.56.1\n\n[2.56.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.55.4...v2.56.0\n\n[2.55.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.55.3...v2.55.4\n\n[2.55.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.55.2...v2.55.3\n\n[2.55.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.55.1...v2.55.2\n\n[2.55.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.55.0...v2.55.1\n\n[2.55.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.54.3...v2.55.0\n\n[2.54.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.54.2...v2.54.3\n\n[2.54.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.54.1...v2.54.2\n\n[2.54.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.54.0...v2.54.1\n\n[2.54.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.53.2...v2.54.0\n\n[2.53.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.53.1...v2.53.2\n\n[2.53.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.53.0...v2.53.1\n\n[2.53.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.8...v2.53.0\n\n[2.52.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.7...v2.52.8\n\n[2.52.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.6...v2.52.7\n\n[2.52.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.5...v2.52.6\n\n[2.52.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.4...v2.52.5\n\n[2.52.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.3...v2.52.4\n\n[2.52.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.2...v2.52.3\n\n[2.52.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.1...v2.52.2\n\n[2.52.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.0...v2.52.1\n\n[2.52.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.51.3...v2.52.0\n\n[2.51.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.51.2...v2.51.3\n\n[2.51.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.51.1...v2.51.2\n\n[2.51.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.51.0...v2.51.1\n\n[2.51.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.10...v2.51.0\n\n[2.50.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.9...v2.50.10\n\n[2.50.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.8...v2.50.9\n\n[2.50.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.7...v2.50.8\n\n[2.50.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.6...v2.50.7\n\n[2.50.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.5...v2.50.6\n\n[2.50.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.4...v2.50.5\n\n[2.50.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.3...v2.50.4\n\n[2.50.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.2...v2.50.3\n\n[2.50.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.1...v2.50.2\n\n[2.50.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.0...v2.50.1\n\n[2.50.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.50...v2.50.0\n\n[2.49.50]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.49...v2.49.50\n\n[2.49.49]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.48...v2.49.49\n\n[2.49.48]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.47...v2.49.48\n\n[2.49.47]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.46...v2.49.47\n\n[2.49.46]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.45...v2.49.46\n\n[2.49.45]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.44...v2.49.45\n\n[2.49.44]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.43...v2.49.44\n\n[2.49.43]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.42...v2.49.43\n\n[2.49.42]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.41...v2.49.42\n\n[2.49.41]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.40...v2.49.41\n\n[2.49.40]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.39...v2.49.40\n\n[2.49.39]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.38...v2.49.39\n\n[2.49.38]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.37...v2.49.38\n\n[2.49.37]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.36...v2.49.37\n\n[2.49.36]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.35...v2.49.36\n\n[2.49.35]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.34...v2.49.35\n\n[2.49.34]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.33...v2.49.34\n\n[2.49.33]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.32...v2.49.33\n\n[2.49.32]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.31...v2.49.32\n\n[2.49.31]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.30...v2.49.31\n\n[2.49.30]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.29...v2.49.30\n\n[2.49.29]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.28...v2.49.29\n\n[2.49.28]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.27...v2.49.28\n\n[2.49.27]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.26...v2.49.27\n\n[2.49.26]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.25...v2.49.26\n\n[2.49.25]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.24...v2.49.25\n\n[2.49.24]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.23...v2.49.24\n\n[2.49.23]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.22...v2.49.23\n\n[2.49.22]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.21...v2.49.22\n\n[2.49.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.20...v2.49.21\n\n[2.49.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.19...v2.49.20\n\n[2.49.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.18...v2.49.19\n\n[2.49.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.17...v2.49.18\n\n[2.49.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.16...v2.49.17\n\n[2.49.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.15...v2.49.16\n\n[2.49.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.14...v2.49.15\n\n[2.49.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.13...v2.49.14\n\n[2.49.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.12...v2.49.13\n\n[2.49.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.11...v2.49.12\n\n[2.49.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.10...v2.49.11\n\n[2.49.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.9...v2.49.10\n\n[2.49.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.8...v2.49.9\n\n[2.49.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.7...v2.49.8\n\n[2.49.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.6...v2.49.7\n\n[2.49.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.5...v2.49.6\n\n[2.49.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.4...v2.49.5\n\n[2.49.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.3...v2.49.4\n\n[2.49.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.2...v2.49.3\n\n[2.49.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.1...v2.49.2\n\n[2.49.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.0...v2.49.1\n\n[2.49.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.22...v2.49.0\n\n[2.48.22]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.21...v2.48.22\n\n[2.48.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.20...v2.48.21\n\n[2.48.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.19...v2.48.20\n\n[2.48.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.18...v2.48.19\n\n[2.48.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.17...v2.48.18\n\n[2.48.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.16...v2.48.17\n\n[2.48.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.15...v2.48.16\n\n[2.48.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.14...v2.48.15\n\n[2.48.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.13...v2.48.14\n\n[2.48.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.12...v2.48.13\n\n[2.48.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.11...v2.48.12\n\n[2.48.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.10...v2.48.11\n\n[2.48.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.9...v2.48.10\n\n[2.48.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.8...v2.48.9\n\n[2.48.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.7...v2.48.8\n\n[2.48.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.6...v2.48.7\n\n[2.48.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.5...v2.48.6\n\n[2.48.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.4...v2.48.5\n\n[2.48.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.3...v2.48.4\n\n[2.48.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.2...v2.48.3\n\n[2.48.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.1...v2.48.2\n\n[2.48.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.0...v2.48.1\n\n[2.48.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.32...v2.48.0\n\n[2.47.32]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.31...v2.47.32\n\n[2.47.31]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.30...v2.47.31\n\n[2.47.30]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.29...v2.47.30\n\n[2.47.29]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.28...v2.47.29\n\n[2.47.28]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.27...v2.47.28\n\n[2.47.27]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.26...v2.47.27\n\n[2.47.26]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.25...v2.47.26\n\n[2.47.25]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.24...v2.47.25\n\n[2.47.24]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.23...v2.47.24\n\n[2.47.23]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.22...v2.47.23\n\n[2.47.22]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.21...v2.47.22\n\n[2.47.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.20...v2.47.21\n\n[2.47.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.19...v2.47.20\n\n[2.47.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.18...v2.47.19\n\n[2.47.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.17...v2.47.18\n\n[2.47.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.16...v2.47.17\n\n[2.47.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.15...v2.47.16\n\n[2.47.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.14...v2.47.15\n\n[2.47.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.13...v2.47.14\n\n[2.47.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.12...v2.47.13\n\n[2.47.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.11...v2.47.12\n\n[2.47.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.10...v2.47.11\n\n[2.47.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.9...v2.47.10\n\n[2.47.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.8...v2.47.9\n\n[2.47.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.7...v2.47.8\n\n[2.47.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.6...v2.47.7\n\n[2.47.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.5...v2.47.6\n\n[2.47.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.4...v2.47.5\n\n[2.47.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.3...v2.47.4\n\n[2.47.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.2...v2.47.3\n\n[2.47.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.1...v2.47.2\n\n[2.47.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.0...v2.47.1\n\n[2.47.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.20...v2.47.0\n\n[2.46.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.19...v2.46.20\n\n[2.46.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.18...v2.46.19\n\n[2.46.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.17...v2.46.18\n\n[2.46.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.16...v2.46.17\n\n[2.46.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.15...v2.46.16\n\n[2.46.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.14...v2.46.15\n\n[2.46.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.13...v2.46.14\n\n[2.46.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.12...v2.46.13\n\n[2.46.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.11...v2.46.12\n\n[2.46.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.10...v2.46.11\n\n[2.46.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.9...v2.46.10\n\n[2.46.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.8...v2.46.9\n\n[2.46.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.7...v2.46.8\n\n[2.46.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.6...v2.46.7\n\n[2.46.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.5...v2.46.6\n\n[2.46.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.4...v2.46.5\n\n[2.46.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.3...v2.46.4\n\n[2.46.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.2...v2.46.3\n\n[2.46.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.1...v2.46.2\n\n[2.46.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.0...v2.46.1\n\n[2.46.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.15...v2.46.0\n\n[2.45.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.14...v2.45.15\n\n[2.45.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.13...v2.45.14\n\n[2.45.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.12...v2.45.13\n\n[2.45.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.11...v2.45.12\n\n[2.45.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.10...v2.45.11\n\n[2.45.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.9...v2.45.10\n\n[2.45.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.8...v2.45.9\n\n[2.45.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.7...v2.45.8\n\n[2.45.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.6...v2.45.7\n\n[2.45.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.5...v2.45.6\n\n[2.45.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.4...v2.45.5\n\n[2.45.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.3...v2.45.4\n\n[2.45.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.2...v2.45.3\n\n[2.45.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.1...v2.45.2\n\n[2.45.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.0...v2.45.1\n\n[2.45.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.72...v2.45.0\n\n[2.44.72]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.71...v2.44.72\n\n[2.44.71]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.70...v2.44.71\n\n[2.44.70]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.69...v2.44.70\n\n[2.44.69]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.68...v2.44.69\n\n[2.44.68]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.67...v2.44.68\n\n[2.44.67]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.66...v2.44.67\n\n[2.44.66]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.65...v2.44.66\n\n[2.44.65]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.64...v2.44.65\n\n[2.44.64]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.63...v2.44.64\n\n[2.44.63]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.62...v2.44.63\n\n[2.44.62]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.61...v2.44.62\n\n[2.44.61]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.60...v2.44.61\n\n[2.44.60]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.59...v2.44.60\n\n[2.44.59]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.58...v2.44.59\n\n[2.44.58]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.57...v2.44.58\n\n[2.44.57]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.56...v2.44.57\n\n[2.44.56]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.55...v2.44.56\n\n[2.44.55]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.54...v2.44.55\n\n[2.44.54]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.53...v2.44.54\n\n[2.44.53]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.52...v2.44.53\n\n[2.44.52]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.51...v2.44.52\n\n[2.44.51]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.50...v2.44.51\n\n[2.44.50]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.49...v2.44.50\n\n[2.44.49]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.48...v2.44.49\n\n[2.44.48]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.47...v2.44.48\n\n[2.44.47]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.46...v2.44.47\n\n[2.44.46]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.45...v2.44.46\n\n[2.44.45]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.44...v2.44.45\n\n[2.44.44]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.43...v2.44.44\n\n[2.44.43]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.42...v2.44.43\n\n[2.44.42]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.41...v2.44.42\n\n[2.44.41]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.40...v2.44.41\n\n[2.44.40]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.39...v2.44.40\n\n[2.44.39]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.38...v2.44.39\n\n[2.44.38]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.37...v2.44.38\n\n[2.44.37]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.36...v2.44.37\n\n[2.44.36]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.35...v2.44.36\n\n[2.44.35]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.34...v2.44.35\n\n[2.44.34]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.33...v2.44.34\n\n[2.44.33]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.32...v2.44.33\n\n[2.44.32]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.31...v2.44.32\n\n[2.44.31]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.30...v2.44.31\n\n[2.44.30]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.29...v2.44.30\n\n[2.44.29]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.28...v2.44.29\n\n[2.44.28]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.27...v2.44.28\n\n[2.44.27]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.26...v2.44.27\n\n[2.44.26]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.25...v2.44.26\n\n[2.44.25]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.24...v2.44.25\n\n[2.44.24]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.23...v2.44.24\n\n[2.44.23]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.22...v2.44.23\n\n[2.44.22]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.21...v2.44.22\n\n[2.44.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.20...v2.44.21\n\n[2.44.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.19...v2.44.20\n\n[2.44.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.18...v2.44.19\n\n[2.44.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.17...v2.44.18\n\n[2.44.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.16...v2.44.17\n\n[2.44.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.15...v2.44.16\n\n[2.44.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.14...v2.44.15\n\n[2.44.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.13...v2.44.14\n\n[2.44.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.12...v2.44.13\n\n[2.44.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.11...v2.44.12\n\n[2.44.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.10...v2.44.11\n\n[2.44.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.9...v2.44.10\n\n[2.44.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.8...v2.44.9\n\n[2.44.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.7...v2.44.8\n\n[2.44.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.6...v2.44.7\n\n[2.44.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.5...v2.44.6\n\n[2.44.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.4...v2.44.5\n\n[2.44.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.3...v2.44.4\n\n[2.44.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.2...v2.44.3\n\n[2.44.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.1...v2.44.2\n\n[2.44.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.0...v2.44.1\n\n[2.44.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.43.7...v2.44.0\n\n[2.43.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.43.6...v2.43.7\n\n[2.43.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.43.5...v2.43.6\n\n[2.43.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.43.4...v2.43.5\n\n[2.43.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.43.3...v2.43.4\n\n[2.43.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.43.2...v2.43.3\n\n[2.43.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.43.1...v2.43.2\n\n[2.43.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.43.0...v2.43.1\n\n[2.43.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.42...v2.43.0\n\n[2.42.42]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.41...v2.42.42\n\n[2.42.41]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.40...v2.42.41\n\n[2.42.40]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.39...v2.42.40\n\n[2.42.39]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.38...v2.42.39\n\n[2.42.38]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.37...v2.42.38\n\n[2.42.37]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.36...v2.42.37\n\n[2.42.36]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.35...v2.42.36\n\n[2.42.35]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.34...v2.42.35\n\n[2.42.34]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.33...v2.42.34\n\n[2.42.33]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.32...v2.42.33\n\n[2.42.32]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.31...v2.42.32\n\n[2.42.31]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.30...v2.42.31\n\n[2.42.30]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.29...v2.42.30\n\n[2.42.29]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.28...v2.42.29\n\n[2.42.28]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.27...v2.42.28\n\n[2.42.27]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.26...v2.42.27\n\n[2.42.26]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.25...v2.42.26\n\n[2.42.25]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.24...v2.42.25\n\n[2.42.24]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.23...v2.42.24\n\n[2.42.23]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.22...v2.42.23\n\n[2.42.22]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.21...v2.42.22\n\n[2.42.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.20...v2.42.21\n\n[2.42.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.19...v2.42.20\n\n[2.42.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.18...v2.42.19\n\n[2.42.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.17...v2.42.18\n\n[2.42.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.16...v2.42.17\n\n[2.42.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.15...v2.42.16\n\n[2.42.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.14...v2.42.15\n\n[2.42.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.13...v2.42.14\n\n[2.42.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.12...v2.42.13\n\n[2.42.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.11...v2.42.12\n\n[2.42.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.10...v2.42.11\n\n[2.42.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.9...v2.42.10\n\n[2.42.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.8...v2.42.9\n\n[2.42.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.7...v2.42.8\n\n[2.42.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.6...v2.42.7\n\n[2.42.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.5...v2.42.6\n\n[2.42.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.4...v2.42.5\n\n[2.42.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.3...v2.42.4\n\n[2.42.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.2...v2.42.3\n\n[2.42.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.1...v2.42.2\n\n[2.42.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.0...v2.42.1\n\n[2.42.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.18...v2.42.0\n\n[2.41.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.17...v2.41.18\n\n[2.41.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.16...v2.41.17\n\n[2.41.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.15...v2.41.16\n\n[2.41.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.14...v2.41.15\n\n[2.41.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.13...v2.41.14\n\n[2.41.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.12...v2.41.13\n\n[2.41.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.11...v2.41.12\n\n[2.41.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.10...v2.41.11\n\n[2.41.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.9...v2.41.10\n\n[2.41.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.8...v2.41.9\n\n[2.41.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.7...v2.41.8\n\n[2.41.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.6...v2.41.7\n\n[2.41.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.5...v2.41.6\n\n[2.41.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.4...v2.41.5\n\n[2.41.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.3...v2.41.4\n\n[2.41.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.2...v2.41.3\n\n[2.41.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.1...v2.41.2\n\n[2.41.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.0...v2.41.1\n\n[2.41.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.40.2...v2.41.0\n\n[2.40.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.40.1...v2.40.2\n\n[2.40.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.40.0...v2.40.1\n\n[2.40.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.39.2...v2.40.0\n\n[2.39.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.39.1...v2.39.2\n\n[2.39.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.39.0...v2.39.1\n\n[2.39.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.38.7...v2.39.0\n\n[2.38.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.38.6...v2.38.7\n\n[2.38.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.38.5...v2.38.6\n\n[2.38.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.38.4...v2.38.5\n\n[2.38.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.38.3...v2.38.4\n\n[2.38.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.38.2...v2.38.3\n\n[2.38.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.38.1...v2.38.2\n\n[2.38.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.38.0...v2.38.1\n\n[2.38.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.37.0...v2.38.0\n\n[2.37.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.36.0...v2.37.0\n\n[2.36.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.35.0...v2.36.0\n\n[2.35.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.34.3...v2.35.0\n\n[2.34.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.34.2...v2.34.3\n\n[2.34.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.34.1...v2.34.2\n\n[2.34.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.34.0...v2.34.1\n\n[2.34.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.36...v2.34.0\n\n[2.33.36]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.35...v2.33.36\n\n[2.33.35]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.34...v2.33.35\n\n[2.33.34]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.33...v2.33.34\n\n[2.33.33]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.32...v2.33.33\n\n[2.33.32]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.31...v2.33.32\n\n[2.33.31]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.30...v2.33.31\n\n[2.33.30]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.29...v2.33.30\n\n[2.33.29]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.28...v2.33.29\n\n[2.33.28]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.27...v2.33.28\n\n[2.33.27]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.26...v2.33.27\n\n[2.33.26]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.25...v2.33.26\n\n[2.33.25]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.24...v2.33.25\n\n[2.33.24]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.23...v2.33.24\n\n[2.33.23]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.22...v2.33.23\n\n[2.33.22]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.21...v2.33.22\n\n[2.33.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.20...v2.33.21\n\n[2.33.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.19...v2.33.20\n\n[2.33.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.18...v2.33.19\n\n[2.33.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.17...v2.33.18\n\n[2.33.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.16...v2.33.17\n\n[2.33.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.15...v2.33.16\n\n[2.33.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.14...v2.33.15\n\n[2.33.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.13...v2.33.14\n\n[2.33.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.12...v2.33.13\n\n[2.33.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.11...v2.33.12\n\n[2.33.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.10...v2.33.11\n\n[2.33.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.9...v2.33.10\n\n[2.33.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.8...v2.33.9\n\n[2.33.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.7...v2.33.8\n\n[2.33.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.6...v2.33.7\n\n[2.33.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.5...v2.33.6\n\n[2.33.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.4...v2.33.5\n\n[2.33.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.3...v2.33.4\n\n[2.33.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.2...v2.33.3\n\n[2.33.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.1...v2.33.2\n\n[2.33.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.0...v2.33.1\n\n[2.33.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.20...v2.33.0\n\n[2.32.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.19...v2.32.20\n\n[2.32.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.18...v2.32.19\n\n[2.32.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.17...v2.32.18\n\n[2.32.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.16...v2.32.17\n\n[2.32.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.15...v2.32.16\n\n[2.32.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.14...v2.32.15\n\n[2.32.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.13...v2.32.14\n\n[2.32.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.12...v2.32.13\n\n[2.32.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.11...v2.32.12\n\n[2.32.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.10...v2.32.11\n\n[2.32.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.9...v2.32.10\n\n[2.32.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.8...v2.32.9\n\n[2.32.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.7...v2.32.8\n\n[2.32.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.6...v2.32.7\n\n[2.32.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.5...v2.32.6\n\n[2.32.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.4...v2.32.5\n\n[2.32.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.3...v2.32.4\n\n[2.32.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.2...v2.32.3\n\n[2.32.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.1...v2.32.2\n\n[2.32.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.0...v2.32.1\n\n[2.32.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.31.3...v2.32.0\n\n[2.31.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.31.2...v2.31.3\n\n[2.31.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.31.1...v2.31.2\n\n[2.31.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.31.0...v2.31.1\n\n[2.31.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.30.0...v2.31.0\n\n[2.30.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.29.8...v2.30.0\n\n[2.29.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.29.7...v2.29.8\n\n[2.29.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.29.6...v2.29.7\n\n[2.29.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.29.5...v2.29.6\n\n[2.29.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.29.4...v2.29.5\n\n[2.29.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.29.3...v2.29.4\n\n[2.29.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.29.2...v2.29.3\n\n[2.29.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.29.1...v2.29.2\n\n[2.29.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.29.0...v2.29.1\n\n[2.29.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.16...v2.29.0\n\n[2.28.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.15...v2.28.16\n\n[2.28.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.14...v2.28.15\n\n[2.28.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.13...v2.28.14\n\n[2.28.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.12...v2.28.13\n\n[2.28.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.11...v2.28.12\n\n[2.28.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.10...v2.28.11\n\n[2.28.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.9...v2.28.10\n\n[2.28.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.8...v2.28.9\n\n[2.28.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.7...v2.28.8\n\n[2.28.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.6...v2.28.7\n\n[2.28.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.5...v2.28.6\n\n[2.28.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.4...v2.28.5\n\n[2.28.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.3...v2.28.4\n\n[2.28.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.2...v2.28.3\n\n[2.28.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.1...v2.28.2\n\n[2.28.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.0...v2.28.1\n\n[2.28.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.15...v2.28.0\n\n[2.27.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.14...v2.27.15\n\n[2.27.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.13...v2.27.14\n\n[2.27.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.12...v2.27.13\n\n[2.27.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.11...v2.27.12\n\n[2.27.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.10...v2.27.11\n\n[2.27.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.9...v2.27.10\n\n[2.27.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.8...v2.27.9\n\n[2.27.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.7...v2.27.8\n\n[2.27.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.6...v2.27.7\n\n[2.27.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.5...v2.27.6\n\n[2.27.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.4...v2.27.5\n\n[2.27.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.3...v2.27.4\n\n[2.27.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.2...v2.27.3\n\n[2.27.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.1...v2.27.2\n\n[2.27.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.0...v2.27.1\n\n[2.27.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.20...v2.27.0\n\n[2.26.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.19...v2.26.20\n\n[2.26.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.18...v2.26.19\n\n[2.26.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.17...v2.26.18\n\n[2.26.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.16...v2.26.17\n\n[2.26.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.15...v2.26.16\n\n[2.26.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.14...v2.26.15\n\n[2.26.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.13...v2.26.14\n\n[2.26.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.12...v2.26.13\n\n[2.26.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.11...v2.26.12\n\n[2.26.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.10...v2.26.11\n\n[2.26.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.9...v2.26.10\n\n[2.26.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.8...v2.26.9\n\n[2.26.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.7...v2.26.8\n\n[2.26.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.6...v2.26.7\n\n[2.26.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.5...v2.26.6\n\n[2.26.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.4...v2.26.5\n\n[2.26.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.3...v2.26.4\n\n[2.26.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.2...v2.26.3\n\n[2.26.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.1...v2.26.2\n\n[2.26.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.0...v2.26.1\n\n[2.26.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.25.11...v2.26.0\n\n[2.25.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.25.10...v2.25.11\n\n[2.25.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.25.9...v2.25.10\n\n[2.25.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.25.8...v2.25.9\n\n[2.25.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.25.7...v2.25.8\n\n[2.25.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.25.6...v2.25.7\n\n[2.25.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.25.5...v2.25.6\n\n[2.25.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.25.4...v2.25.5\n\n[2.25.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.25.3...v2.25.4\n\n[2.25.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.25.2...v2.25.3\n\n[2.25.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.25.1...v2.25.2\n\n[2.25.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.25.0...v2.25.1\n\n[2.25.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.24.4...v2.25.0\n\n[2.24.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.24.3...v2.24.4\n\n[2.24.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.24.2...v2.24.3\n\n[2.24.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.24.1...v2.24.2\n\n[2.24.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.24.0...v2.24.1\n\n[2.24.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.23.9...v2.24.0\n\n[2.23.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.23.8...v2.23.9\n\n[2.23.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.23.7...v2.23.8\n\n[2.23.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.23.6...v2.23.7\n\n[2.23.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.23.5...v2.23.6\n\n[2.23.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.23.4...v2.23.5\n\n[2.23.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.23.3...v2.23.4\n\n[2.23.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.23.2...v2.23.3\n\n[2.23.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.23.1...v2.23.2\n\n[2.23.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.23.0...v2.23.1\n\n[2.23.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.22.10...v2.23.0\n\n[2.22.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.22.9...v2.22.10\n\n[2.22.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.22.8...v2.22.9\n\n[2.22.8]: https://redirect.github.com/taiki-e/ins\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - \"before 8am on Monday\" (UTC),\nAutomerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n👻 **Immortal**: This PR will be recreated if closed unmerged. Get\n[config\nhelp](https://redirect.github.com/renovatebot/renovate/discussions) if\nthat's undesired.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0MS4xNTYuMSIsInVwZGF0ZWRJblZlciI6IjQxLjE1OS40IiwidGFyZ2V0QnJhbmNoIjoibWFpbiIsImxhYmVscyI6WyJkZXBlbmRlbmNpZXMiXX0=-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2025-10-28T06:49:11Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/f71769d880ddce56b4525bc88d709d9a7edec65a"
        },
        "date": 1761646442556,
        "tool": "customSmallerIsBetter",
        "benches": [
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
            "value": 16.974477745898113,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 19.424863660969763,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 28.796875,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 29.01171875,
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
            "value": 875515.4044546917,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 12964383.737766394,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - Network Utilization"
          },
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
            "value": 16.04196665483545,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 16.17542022131084,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.2484375,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.359375,
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
            "value": 293909.73777561163,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13100523.19422759,
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
            "value": 12.752295851631953,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 12.848116503290747,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 21.021484375,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 21.12109375,
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
            "value": 543002.222955754,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13129894.224885518,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - Network Utilization"
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
            "value": 19.50873268136444,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 19.86319159842702,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 21.464453125,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 21.53515625,
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
            "value": 335771.6639380107,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 12999384.31440952,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Network Utilization"
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
          "id": "4165b8b384840c89e3eab8cf7b44694b293d3975",
          "message": "[query-engine] Improve accessor diagnostics\\logging in recordset engine (#1344)\n\n## Details\n\nThe goal here is to re-parent diagnostics for accessors to the root\nexpression instead of the selector.\n\nThat turns something like this:\n```\n                  |               | [Verbose] ScalarExpression(GetType): Evaluated as: 'String'\n                  |               | [Verbose] ScalarExpression(GetType): Resolved 'string' value for key 'String' specified in accessor expression\n                  |               | [Verbose] ScalarExpression(Constant): Resolved constant with id '0' on line 2 at column 27 as: string\n```\n\nInto:\n```\n                  |               | [Verbose] ScalarExpression(Constant): Resolved 'Map' constant with id '0' defined on line 2 at column 27\n                  |               | [Verbose] ScalarExpression(GetType): Evaluated as: 'String'\n                  |               | [Verbose] ScalarExpression(Constant): Resolved 'string' value for key 'String' specified in accessor expression\n                  |               | [Verbose] ScalarExpression(Constant): Evaluated as: 'string'\n```\n\nI know it is still hard to read but I think it is an improvement 🤣\n\n### Full example\n\nBefore:\n\n```\nln   1: source\nln   2:  | extend EventNameType = gettype(EventName)\n                  |               |       | [Verbose] StaticScalar(String): Evaluated as: 'EventName'\n                  |               |       | [Verbose] StaticScalar(String): Resolved 'event_name' value for key 'EventName' specified in accessor expression\n                  |               |       | [Verbose] ScalarExpression(Source): Evaluated as: 'event_name'\n                  |               | [Verbose] ScalarExpression(GetType): Evaluated as: 'String'\n                  |               | [Verbose] ScalarExpression(GetType): Resolved 'string' value for key 'String' specified in accessor expression\n                  |               | [Verbose] ScalarExpression(Constant): Resolved constant with id '0' on line 2 at column 27 as: string\n                  | [Verbose] StaticScalar(String): Evaluated as: 'Attributes'\n                  | [Verbose] StaticScalar(String): Evaluated as: 'EventNameType'\n                  | [Verbose] StaticScalar(String): Resolved 'Map' value for key 'Attributes' specified in accessor expression\n                  | [Verbose] MutableValueExpression(Source): Evaluated as: Map write for 'EventNameType' key\n                  | [Verbose] SetTransformExpression: Map key 'EventNameType' created on target map\n```\n\nAfter:\n```\nln   1: source\nln   2:  | extend EventNameType = gettype(EventName)\n                  |               |       | [Verbose] StaticScalar(String): Evaluated as: 'EventName'\n                  |               |       | [Verbose] ScalarExpression(Source): Resolved 'event_name' value for key 'EventName' specified in accessor expression\n                  |               |       | [Verbose] ScalarExpression(Source): Evaluated as: 'event_name'\n                  |               | [Verbose] ScalarExpression(Constant): Resolved 'Map' constant with id '0' defined on line 2 at column 27\n                  |               | [Verbose] ScalarExpression(GetType): Evaluated as: 'String'\n                  |               | [Verbose] ScalarExpression(Constant): Resolved 'string' value for key 'String' specified in accessor expression\n                  |               | [Verbose] ScalarExpression(Constant): Evaluated as: 'string'\n                  | [Verbose] StaticScalar(String): Evaluated as: 'Attributes'\n                  | [Verbose] StaticScalar(String): Evaluated as: 'EventNameType'\n                  | [Verbose] MutableValueExpression(Source): Resolved 'Map' value for key 'Attributes' specified in accessor expression\n                  | [Verbose] MutableValueExpression(Source): Evaluated as: Map write for 'EventNameType' key\n                  | [Verbose] SetTransformExpression: Map key 'EventNameType' created on target map\n```",
          "timestamp": "2025-10-28T22:19:06Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/4165b8b384840c89e3eab8cf7b44694b293d3975"
        },
        "date": 1761732869507,
        "tool": "customSmallerIsBetter",
        "benches": [
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
            "value": 20.412107380494387,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 20.760850096004955,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 26.84921875,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 26.9453125,
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
            "value": 337593.0559647594,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 12936509.33852703,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Network Utilization"
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
            "value": 13.134310003458122,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 13.40747688162886,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 24.690234375,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 24.84765625,
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
            "value": 544564.8668412204,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13191722.706801545,
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
            "value": 16.640414176474458,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 16.766260015540016,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 25.0859375,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 25.29296875,
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
            "value": 880635.1442798001,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13015878.855272148,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - Network Utilization"
          },
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
            "value": 14.202424543904888,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 14.59218854875107,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 26.821484375,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 27.01171875,
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
            "value": 294524.17534506833,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13164848.6165115,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - Network Utilization"
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
          "id": "71bc65ad49125e3d320b2b118c74034b725f5f03",
          "message": "[otap-df-otap] Use a better name for OTAP Receiver config setting (#1365)\n\nRelated to\nhttps://github.com/open-telemetry/otel-arrow/pull/1356#discussion_r2474120888\n\n## Changes\n- Rename `message_size` to `response_stream_channel_size` to better\nreflect the purpose of this setting",
          "timestamp": "2025-10-30T00:53:51Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/71bc65ad49125e3d320b2b118c74034b725f5f03"
        },
        "date": 1761819260668,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 125200,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 100,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 9.39515318600141,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 9.643427706621182,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 80.815625,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 115.11328125,
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
            "name": "network_tx_bytes_rate_avg",
            "value": 81446.15737018926,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13118448.613816496,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 125200,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 100,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 11.198101654303768,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 11.788144087986987,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 98.26875,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 143.78515625,
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
            "name": "network_tx_bytes_rate_avg",
            "value": 81598.2093680345,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 12952135.322771804,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Network Utilization"
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
            "value": 12.58336124751861,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 12.73278520145387,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 24.756640625,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 24.921875,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 125100,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 125100,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 541799.4728398165,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13189992.422657378,
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
            "value": 16.583369489618782,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 17.20941973516124,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 23.576171875,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.765625,
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
            "value": 877349.0575961478,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13027389.078252848,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - Network Utilization"
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
          "id": "aa23e7134b557caae0b55a623e61a2aae5d9058e",
          "message": "Fix continuous benchmarks for OTAP Receiver (#1367)\n\nFollow-up to #1365 \n\n## Changes\n- Update the continuous benchmark config files for OTAP receiver to use\nthe updated setting name\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2025-10-30T23:26:47Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/aa23e7134b557caae0b55a623e61a2aae5d9058e"
        },
        "date": 1761905737050,
        "tool": "customSmallerIsBetter",
        "benches": [
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
            "value": 16.692810000197124,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 16.80003134531928,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 26.97890625,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 27.1484375,
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
            "value": 878192.496060725,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13018578.881055813,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - Network Utilization"
          },
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
            "value": 15.029490179950894,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 15.433513756760933,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 21.44921875,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 21.55078125,
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
            "value": 293992.27320586576,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13097020.418726599,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - Network Utilization"
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
            "value": 19.593734519921956,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 20.1670143327409,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 23.377734375,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.5625,
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
            "value": 335082.71664308547,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 12992160.573958796,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Network Utilization"
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
            "value": 12.94211265455484,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 13.157998345513915,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.05859375,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.13671875,
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
            "value": 541412.6620097393,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13186182.177753199,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - Network Utilization"
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
          "id": "a9d4a890b979417cf51314c9835237877bb9936e",
          "message": "[query-engine] Recordset engine diagnostics/logging improvements (#1369)\n\n## Changes\n\n* Clean up the code around \"Resolved as:\" logging to have less\nduplication\n* Maps and Arrays are no longer logged as JSON strings (to prevent\nbloat)\n* Strings will only be logged up to 32 characters (to prevent bloat)\n* Add type information. Instead of `'null'` (which could be a `null`\nvalue or a string of `\"null\"`) we will now get `[Null]` or\n`[String(\"null\")]`.",
          "timestamp": "2025-10-31T17:54:48Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/a9d4a890b979417cf51314c9835237877bb9936e"
        },
        "date": 1761992005391,
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
            "value": 16.088044046862777,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 16.37461564947654,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 23.442578125,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.50390625,
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
            "value": 297262.71535951516,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13098717.648884024,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - Network Utilization"
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
            "value": 18.522872756156314,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 18.817082291731186,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 25.379296875,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 25.5546875,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 125100,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 125100,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 335982.3958898025,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 12993936.624231426,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Network Utilization"
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
            "value": 16.00806604586746,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 16.254944272656854,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 23.5328125,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.64453125,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 125500,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 125500,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 878111.2641841285,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13023463.266573776,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - Network Utilization"
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
            "value": 12.533067565825572,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 12.733896466557656,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.962890625,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 21.21484375,
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
            "value": 537495.9508536776,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13117340.797318224,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - Network Utilization"
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
          "id": "a9d4a890b979417cf51314c9835237877bb9936e",
          "message": "[query-engine] Recordset engine diagnostics/logging improvements (#1369)\n\n## Changes\n\n* Clean up the code around \"Resolved as:\" logging to have less\nduplication\n* Maps and Arrays are no longer logged as JSON strings (to prevent\nbloat)\n* Strings will only be logged up to 32 characters (to prevent bloat)\n* Add type information. Instead of `'null'` (which could be a `null`\nvalue or a string of `\"null\"`) we will now get `[Null]` or\n`[String(\"null\")]`.",
          "timestamp": "2025-10-31T17:54:48Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/a9d4a890b979417cf51314c9835237877bb9936e"
        },
        "date": 1762078371422,
        "tool": "customSmallerIsBetter",
        "benches": [
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
            "value": 22.2488512477056,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 22.66060560006198,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 26.7625,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 26.859375,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 125100,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 125100,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 335580.4897888103,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 12992664.889084246,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Network Utilization"
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
            "value": 13.014218674677098,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 13.22290960303335,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 21.082421875,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 21.2734375,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 125100,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 125100,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 542219.5150386845,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13189939.365368748,
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
            "value": 16.263851465925246,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 16.646722307155592,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 27.226953125,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 27.3828125,
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
            "value": 882949.9761429265,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13021832.851157736,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - Network Utilization"
          },
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
            "value": 14.859983160393034,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 15.0375425348621,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 21.639453125,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 21.71875,
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
            "value": 294319.4817275307,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13100100.825743653,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - Network Utilization"
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
          "id": "a9d4a890b979417cf51314c9835237877bb9936e",
          "message": "[query-engine] Recordset engine diagnostics/logging improvements (#1369)\n\n## Changes\n\n* Clean up the code around \"Resolved as:\" logging to have less\nduplication\n* Maps and Arrays are no longer logged as JSON strings (to prevent\nbloat)\n* Strings will only be logged up to 32 characters (to prevent bloat)\n* Add type information. Instead of `'null'` (which could be a `null`\nvalue or a string of `\"null\"`) we will now get `[Null]` or\n`[String(\"null\")]`.",
          "timestamp": "2025-10-31T17:54:48Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/a9d4a890b979417cf51314c9835237877bb9936e"
        },
        "date": 1762164879137,
        "tool": "customSmallerIsBetter",
        "benches": [
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
            "value": 16.541056437859254,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 16.81920750733138,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 27.15625,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 27.26953125,
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
            "value": 871815.4303100078,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 12952157.020700557,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - Network Utilization"
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
            "value": 13.012320632307189,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 13.199958874505851,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.14375,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.29296875,
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
            "value": 541120.925627383,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13183450.600931551,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - Network Utilization"
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
            "value": 22.79002345383121,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 23.22830632828871,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 23.3796875,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.93359375,
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
            "value": 334019.89367956895,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 12937055.515187858,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Network Utilization"
          },
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
            "value": 16.11258055972778,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 16.69195911609088,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 26.974609375,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 27.0390625,
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
            "value": 297855.437203837,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13096796.277283084,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "clhain",
            "username": "clhain",
            "email": "8164192+clhain@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "b06c055168630b8a3f2ee859f2d62428e82a89de",
          "message": "[PipelinePerf] setup verification of manual workflow runs for PRs (#1381)\n\nThese changes allow automated verification of a manually triggered perf\ntest workflow for a given PR number. Because of how github actions work\nfor this weird case, it's a 2 step thing:\n\n- The new workflow file below (pipeline-perf-verify-pr.yaml) inserts a\nplaceholder status into PRs on create/update that will remain \"pending\"\nindefinitely.\n- The original manual workflow now updates that placeholder status with\nthe result of it's own run (for the specified PR number).\n\nSo the flow is:\n- New PR created, new job inserts the pending status result\n- Maintainer triggers the manual run with target PR number\n- Manual job runs and sets the previously pending result to pass/fail\n\nNew commits will clear the result back to pending.",
          "timestamp": "2025-11-04T22:41:54Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/b06c055168630b8a3f2ee859f2d62428e82a89de"
        },
        "date": 1762338001452,
        "tool": "customSmallerIsBetter",
        "benches": [
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
            "value": 16.433092488800717,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 16.62294535769588,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 25.446484375,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 25.6796875,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 125100,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 125100,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 879976.6086780261,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13030609.589166548,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - Network Utilization"
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
            "value": 13.09678506954142,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 13.371455361040166,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 24.78671875,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 24.96875,
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
            "value": 540392.3769498774,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13137357.922217343,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - Network Utilization"
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
            "value": 21.518253930555613,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 21.754411197896527,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 25.00078125,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 25.10546875,
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
            "value": 334382.2200062574,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13004440.852099942,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Network Utilization"
          },
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
            "value": 14.01683170949594,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 15.022799074502547,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 27.03125,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 28.90625,
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
            "value": 299516.4147456842,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13172154.381723292,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - Network Utilization"
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
          "id": "e9fc2a4d40ea8196454da17a1bfb9dd7c5d2ed8a",
          "message": "[otel-arrow-rust] Replace Snafu with thiserror (#1389)\n\nPart of #1377 #867 \nReplaces snafu with thiserror.\n\nHere's my retrospective on Snafu vs. Thiserror:\n\nSnafu has a little more sugar, maybe too much magic. E.g., options and\nresults both have context extensions, with thiserror you choose\nok_or_else() or map_err() explicitly. While I admire Snafu for its\nautomation, I prefer the more sparing approach taken by thiserror.",
          "timestamp": "2025-11-05T22:16:25Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/e9fc2a4d40ea8196454da17a1bfb9dd7c5d2ed8a"
        },
        "date": 1762424243256,
        "tool": "customSmallerIsBetter",
        "benches": [
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
            "value": 16.5279800384976,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 16.748208584446676,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 21.237109375,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 21.515625,
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
            "value": 879762.9960451608,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13042447.283640763,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - Network Utilization"
          },
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
            "value": 14.333202844646495,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 14.696366669763123,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 26.765625,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 26.89453125,
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
            "value": 294502.49423788895,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13111318.517785605,
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
            "value": 12.892147874168444,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 13.180312142136705,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 22.917578125,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 22.984375,
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
            "value": 545245.3250182394,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13201347.013269586,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - Network Utilization"
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
            "value": 20.32030640458341,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 21.671005074696183,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 25.294921875,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 25.34375,
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
            "value": 337952.41209078603,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 12935870.59194843,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Network Utilization"
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
          "id": "dbd017c4dbf2305c619c31da35ce0722ff46b369",
          "message": "[query-engine] Add special handling for invalid equality expressions in KQL comparisons (#1400)\n\n## Changes\n\n* Add a rule in KQL pest to allow `where [scalar] = [scalar]` so that a\nmore intentional error message can be generated hinting that `==` is the\nmost likely solution",
          "timestamp": "2025-11-06T23:37:57Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/dbd017c4dbf2305c619c31da35ce0722ff46b369"
        },
        "date": 1762514337076,
        "tool": "customSmallerIsBetter",
        "benches": [
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
            "value": 21.41651175859384,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 21.54980241560855,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 25.37734375,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 26.0234375,
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
            "value": 336095.08817886416,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 12999254.542772604,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Network Utilization"
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
            "value": 16.54196652693699,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 17.135829878435928,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 23.12890625,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.20703125,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 125100,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 125100,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 873158.2689604334,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 12957977.717752628,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - Network Utilization"
          },
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
            "value": 15.356243109865291,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 15.690497991374,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 21.511328125,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 21.67578125,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 125100,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 125100,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 293907.5594090472,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13105263.397681596,
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
            "value": 12.916572074397498,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 13.662575785747268,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 13.312890625,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 13.43359375,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 125100,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 125100,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 539301.7684934516,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13121828.73473472,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - Network Utilization"
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
          "id": "183c1a70d3b82a538e6573e9150f61f8b768106f",
          "message": "Update Go collector dependencies v0.139.0 and v1.45.0 (#1401)",
          "timestamp": "2025-11-08T00:05:20Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/183c1a70d3b82a538e6573e9150f61f8b768106f"
        },
        "date": 1762596961139,
        "tool": "customSmallerIsBetter",
        "benches": [
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
            "value": 12.732991961888146,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 12.947644994216088,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 21.11484375,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 21.28125,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 125100,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 125100,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 541560.8078159199,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13185771.971182015,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - Network Utilization"
          },
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
            "value": 16.864196354039297,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 18.770960970408716,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 26.791796875,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 26.88671875,
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
            "value": 292856.8945305962,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13107846.91057025,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - Network Utilization"
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
            "value": 23.252239319627204,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 23.497983246300457,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 22.830859375,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 22.92578125,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 125100,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 125100,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 335111.10789534636,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 12998181.719116416,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Network Utilization"
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
            "value": 16.61716730931777,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 16.897646450813323,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 26.9703125,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 27,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 125100,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 125100,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 883061.0569511164,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13024427.39018245,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - Network Utilization"
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
          "id": "183c1a70d3b82a538e6573e9150f61f8b768106f",
          "message": "Update Go collector dependencies v0.139.0 and v1.45.0 (#1401)",
          "timestamp": "2025-11-08T00:05:20Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/183c1a70d3b82a538e6573e9150f61f8b768106f"
        },
        "date": 1762683365154,
        "tool": "customSmallerIsBetter",
        "benches": [
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
            "value": 22.16977534612402,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 22.571021491514866,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 25.405078125,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 25.52734375,
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
            "value": 337811.8305892267,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 12932332.406768462,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Network Utilization"
          },
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
            "value": 15.541182127591991,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 16.941484773662552,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 25.331640625,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 25.44921875,
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
            "value": 298094.4210424415,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13103684.807290198,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - Network Utilization"
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
            "value": 16.588480026079438,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 20.829394001862774,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 27.394921875,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 27.62890625,
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
            "value": 878661.2153802892,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13033424.184280371,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - Network Utilization"
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
            "value": 13.032874112654264,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 13.230799535423927,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 22.824609375,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 22.9765625,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 125100,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 125100,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 546471.5066508718,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13193350.379355874,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - Network Utilization"
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
          "id": "183c1a70d3b82a538e6573e9150f61f8b768106f",
          "message": "Update Go collector dependencies v0.139.0 and v1.45.0 (#1401)",
          "timestamp": "2025-11-08T00:05:20Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/183c1a70d3b82a538e6573e9150f61f8b768106f"
        },
        "date": 1762771069211,
        "tool": "customSmallerIsBetter",
        "benches": [
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
            "value": 17.719764234426183,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 18.71448544411719,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 30.723046875,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 30.890625,
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
            "value": 882799.7610962058,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13018026.736677382,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - Network Utilization"
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
            "value": 12.940250573775353,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 13.235640832752882,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.08515625,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.1640625,
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
            "value": 538686.1040503436,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13114091.303235706,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - Network Utilization"
          },
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
            "value": 16.497197409087676,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 16.991518140975987,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 26.834375,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 26.95703125,
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
            "value": 295535.24393628986,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13159674.921539705,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - Network Utilization"
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
            "value": 22.358663427263167,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 24.687599857640233,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 25.056640625,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 25.22265625,
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
            "value": 335954.7382384245,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13001276.504358802,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Network Utilization"
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
          "id": "ba0400f949aa76100aeed43d4506736a531062be",
          "message": "Use GITHUB_STEP_SUMMARY on release action dry_run (#1412)\n\nInspired by usage in other repositories - nicer to avoid digging through\ndetailed dry run logs to ensure release looks as expected.",
          "timestamp": "2025-11-10T23:40:26Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/ba0400f949aa76100aeed43d4506736a531062be"
        },
        "date": 1762856213436,
        "tool": "customSmallerIsBetter",
        "benches": [
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
            "value": 21.680164900186355,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 21.84667345184727,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 26.871484375,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 27,
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
            "value": 335589.32951516064,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 12992505.011488948,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Network Utilization"
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
            "value": 12.943089582022619,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 13.276348670233387,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.090625,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.484375,
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
            "value": 542834.367911875,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13121003.700001808,
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
            "value": 16.367622393251143,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 16.562604939227374,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 25.218359375,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 25.33203125,
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
            "value": 878802.5593270144,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13024414.863406444,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - Network Utilization"
          },
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
            "value": 13.76577004692942,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 14.104921546343736,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 24.99140625,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 25.12890625,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 125100,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 125100,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 298442.83332004613,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13175042.397663241,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "otelbot[bot]",
            "username": "otelbot[bot]",
            "email": "197425009+otelbot[bot]@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "430a4835157fdbe9cbf98b75970ff971476a39af",
          "message": "chore(release) Prepare Release v0.45.0 (#1415)\n\n## Release v0.45.0\n\nThis PR prepares the repository for release v0.45.0.\n\n### Changes included:\n- Updated CHANGELOG.md with release notes\n- Updated collector/otelarrowcol-build.yaml version to v0.45.0\n- Updated collector/cmd/otelarrowcol/main.go version to v0.45.0\n\n### Release Notes:\n- Upgrade to v0.139.0 / v1.45.0 of collector dependencies.\n[#1401](https://github.com/open-telemetry/otel-arrow/pull/1401)\n\n### Checklist:\n- [x] Verify CHANGELOG.md formatting and content\n- [x] Verify collector version update in\ncollector/otelarrowcol-build.yaml\n- [x] Verify collector main.go version update in\ncollector/cmd/otelarrowcol/main.go\n- [x] Confirm all tests pass\n- [x] Ready to merge and tag release\n\nAfter merging this PR, run the **Push Release** workflow to create git\ntags and publish the GitHub release.\n\nCo-authored-by: otelbot <197425009+otelbot@users.noreply.github.com>",
          "timestamp": "2025-11-11T16:33:50Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/430a4835157fdbe9cbf98b75970ff971476a39af"
        },
        "date": 1762942638363,
        "tool": "customSmallerIsBetter",
        "benches": [
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
            "value": 13.211001451401144,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 13.392595286446204,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 21.06171875,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 21.21484375,
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
            "value": 541784.4670041214,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13189917.700400373,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - Network Utilization"
          },
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
            "value": 16.253604816148563,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 16.75533005558987,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 21.5015625,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 21.546875,
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
            "value": 299293.24266408454,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13164672.094352668,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - Network Utilization"
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
            "value": 21.920311531361786,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 22.230687618162097,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 25.053515625,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 25.20703125,
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
            "value": 334147.8589761707,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 12928200.75845509,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Network Utilization"
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
            "value": 16.767799648278686,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 16.991155312862556,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 28.799609375,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 29.07421875,
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
            "value": 878798.663295658,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13029619.680847256,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - Network Utilization"
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
          "id": "1477cd3640ddf45c083cf9170288010ce185bb45",
          "message": "Support expanded and packed encoding for repeated primitive fields when decoding protobuf (#1424)\n\ncloses #1399 \n\nAccording to the [protobuf encoding\ndocs](https://protobuf.dev/programming-guides/encoding/), there's two\ncases decoders need to handle:\n\n> Protocol buffer parsers must be able to parse repeated fields that\nwere compiled as packed as if they were not packed, and vice versa.\n\nAlso, if the encoding is \"packed\", we need to handle if there are more\nthan one segment of the buffer containing packed values\n> Note that although there’s usually no reason to encode more than one\nkey-value pair for a packed repeated field, parsers must be prepared to\naccept multiple key-value pairs. In this case, the payloads should be\nconcatenated. Each pair must contain a whole number of elements. The\nfollowing is a valid encoding of the same message above that parsers\nmust accept\n\nThis PR brings our decoding logic in line with the recommendations for\nthe few fields we have that could have this encoding. Before this\nchange, we were assuming the encoding was always packed.\n\nThis change implements a new iterator for these types called\n`RepeatedPrimitiveIter` which encapsulates the logic of determining how\nto iterate the buffer and produce values correctly for both types of\nencodings. Internally, it use a combination of the\n`RepeatedFieldProtoBytesParser` and the packed field iterators which\nwere already implemented before this change.\n\n---------\n\nCo-authored-by: Joshua MacDonald <jmacd@users.noreply.github.com>\nCo-authored-by: Utkarsh Umesan Pillai <66651184+utpilla@users.noreply.github.com>",
          "timestamp": "2025-11-12T21:44:53Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/1477cd3640ddf45c083cf9170288010ce185bb45"
        },
        "date": 1763029203239,
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
            "value": 14.534449641435632,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 14.632823728498373,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 21.58828125,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 21.69921875,
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
            "value": 295323.5215327668,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13164810.877482185,
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
            "value": 12.412290343322267,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 13.196709370889112,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 21.293359375,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 21.3828125,
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
            "value": 538695.0645254384,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13123820.232483087,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - Network Utilization"
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
            "value": 22.62630311956142,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 25.269987584278393,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 23.12734375,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.19140625,
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
            "value": 339317.8386791615,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13004217.226897221,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Network Utilization"
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
            "value": 16.56493519123798,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 17.008560799194612,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 23.4078125,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.51171875,
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
            "value": 878954.7442665435,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13024226.956578901,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - Network Utilization"
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
          "id": "cb2f946eed228de75fd07cf4095888bb945b0b21",
          "message": "Rename otap_batch_processor to batch_processor (#1432)\n\nThe additional \"otap_\" in this component name is confusing/unnecessary.\nThe URN is still \"urn:otap:processor:batch\".\n\nThe otap_batch_processor/ subdir had only a metrics struct, moved it\ninto the main code to simplify.",
          "timestamp": "2025-11-14T00:35:41Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/cb2f946eed228de75fd07cf4095888bb945b0b21"
        },
        "date": 1763115445620,
        "tool": "customSmallerIsBetter",
        "benches": [
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
            "value": 20.441634821033684,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 20.703027691355157,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 21.54296875,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 21.65234375,
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
            "value": 335297.961659157,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 12976463.764338229,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Network Utilization"
          },
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
            "value": 15.074466459654975,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 15.558433869245647,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 23.371484375,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.53125,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 125100,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 125100,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 299393.8048483545,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13171876.810391735,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - Network Utilization"
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
            "value": 16.616950569285525,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 16.904944161867704,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 27.1171875,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 27.26953125,
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
            "value": 873886.6117916098,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 12953746.397831341,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - Network Utilization"
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
            "value": 13.151134758109407,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 13.688027343190662,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.98515625,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 21.08203125,
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
            "value": 540143.5684246821,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13127518.193920476,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - Network Utilization"
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
          "id": "e771ca91cbfb6a0158ef31772b6cc9aba68fa044",
          "message": "[query-engine] Fix strcat appending \"null\" bug (#1434)\n\n## Changes\n\n* Fixes recordset engine so that when executing `strcat` it appends\nempty string for `null` instead of \"null\"",
          "timestamp": "2025-11-14T21:38:04Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/e771ca91cbfb6a0158ef31772b6cc9aba68fa044"
        },
        "date": 1763201919487,
        "tool": "customSmallerIsBetter",
        "benches": [
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
            "value": 16.71503136938437,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 16.879453305445814,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 27.283984375,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 27.50390625,
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
            "value": 875656.221032984,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 12971080.872887623,
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
            "value": 21.04586748157721,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 25.030280355349554,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 25.361328125,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 25.44140625,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 125100,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 125100,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 335834.65228016174,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 12996736.016780578,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Network Utilization"
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
            "value": 12.841439974218288,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 13.00940964089891,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 11.4046875,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 11.46875,
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
            "value": 540466.5121271358,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13133951.012989972,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - Network Utilization"
          },
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
            "value": 16.315721090843464,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 22.836219830942227,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 25.338671875,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 25.50390625,
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
            "value": 297850.8880890702,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13101061.590684231,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - Network Utilization"
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
          "id": "e771ca91cbfb6a0158ef31772b6cc9aba68fa044",
          "message": "[query-engine] Fix strcat appending \"null\" bug (#1434)\n\n## Changes\n\n* Fixes recordset engine so that when executing `strcat` it appends\nempty string for `null` instead of \"null\"",
          "timestamp": "2025-11-14T21:38:04Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/e771ca91cbfb6a0158ef31772b6cc9aba68fa044"
        },
        "date": 1763288201944,
        "tool": "customSmallerIsBetter",
        "benches": [
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
            "value": 21.873423141041247,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 22.202594199798497,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 23.71171875,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 25.87890625,
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
            "value": 335201.812960652,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 12993657.47294914,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Network Utilization"
          },
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
            "value": 15.75971726943175,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 16.949237664165828,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 24.943359375,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 25.0546875,
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
            "value": 295066.7460732666,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13163792.091460925,
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
            "value": 12.948337129388154,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 13.336954613543355,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 21.1109375,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 21.23046875,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 125100,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 125100,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 538563.5900897343,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13122345.81877222,
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
            "value": 16.64934984852004,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 17.107287319978344,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 30.739453125,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 30.7890625,
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
            "value": 878071.1212741785,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13023499.586769884,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - Network Utilization"
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
          "id": "e771ca91cbfb6a0158ef31772b6cc9aba68fa044",
          "message": "[query-engine] Fix strcat appending \"null\" bug (#1434)\n\n## Changes\n\n* Fixes recordset engine so that when executing `strcat` it appends\nempty string for `null` instead of \"null\"",
          "timestamp": "2025-11-14T21:38:04Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/e771ca91cbfb6a0158ef31772b6cc9aba68fa044"
        },
        "date": 1763374642321,
        "tool": "customSmallerIsBetter",
        "benches": [
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
            "value": 19.887241855479047,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 20.24270163325335,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 21.78203125,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.62109375,
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
            "value": 335627.7085427134,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13018025.387435023,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Network Utilization"
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
            "value": 13.010395998249397,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 13.387684196518377,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 21.10078125,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 21.30859375,
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
            "value": 544989.7868391364,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13171002.456274614,
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
            "value": 16.520444971705505,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 17.492605275749572,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 27.1875,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 27.28125,
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
            "value": 877889.1309945139,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13003936.050206682,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - Network Utilization"
          },
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
            "value": 16.498622304620728,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 17.276698254888323,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 25.115234375,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 25.265625,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 125100,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 125100,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 294453.8238616254,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13134602.415460384,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - Network Utilization"
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
          "id": "43596412cec500363a1226ad3957237d83fbfcf5",
          "message": "Nit fix to readme on running dataflow engine (#1443)",
          "timestamp": "2025-11-18T00:54:02Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/43596412cec500363a1226ad3957237d83fbfcf5"
        },
        "date": 1763462711848,
        "tool": "customSmallerIsBetter",
        "benches": [
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
            "value": 16.125517549649356,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 16.244018498684415,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 25.030078125,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 25.109375,
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
            "value": 875114.2424631404,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 12953769.736366507,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - Network Utilization"
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
            "value": 13.026395438883995,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 13.713352652934798,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.34375,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.5,
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
            "value": 539984.9663109017,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13135376.373535762,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - Network Utilization"
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
            "value": 24.754333059872476,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 27.065066015908563,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 25.039453125,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 25.2109375,
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
            "value": 335026.9870710278,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 12982430.758506699,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Network Utilization"
          },
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
            "value": 15.849244880201935,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 16.195221497168568,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 27.2296875,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 27.37109375,
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
            "value": 295360.11791971803,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13160828.71211893,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - Network Utilization"
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
          "id": "a2fc3baf6988b2014a28f12006b56757c82ebc37",
          "message": "Fix meeting time (#1450)\n\nLooks like the doc in the repo was outdated.",
          "timestamp": "2025-11-19T00:42:26Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/a2fc3baf6988b2014a28f12006b56757c82ebc37"
        },
        "date": 1763549411639,
        "tool": "customSmallerIsBetter",
        "benches": [
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
            "value": 24.089439910269935,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 24.80474108589575,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 23.491796875,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.61328125,
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
            "value": 333841.78289207665,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 12932118.474674894,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Network Utilization"
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
            "value": 13.472793474483776,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 14.701958142414862,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.30078125,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.41796875,
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
            "value": 544317.6903481963,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13179827.125737425,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - Network Utilization"
          },
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
            "value": 14.489387981284107,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 14.735205884630275,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 23.371484375,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.45703125,
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
            "value": 297611.25686391874,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13086711.49710311,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - Network Utilization"
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
            "value": 17.153259323379693,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 17.490112347017817,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 27.2546875,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 27.47265625,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 125100,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 125100,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 882443.06102056,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13022804.539482588,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - Network Utilization"
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
          "id": "a2fc3baf6988b2014a28f12006b56757c82ebc37",
          "message": "Fix meeting time (#1450)\n\nLooks like the doc in the repo was outdated.",
          "timestamp": "2025-11-19T00:42:26Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/a2fc3baf6988b2014a28f12006b56757c82ebc37"
        },
        "date": 1763634644922,
        "tool": "customSmallerIsBetter",
        "benches": [
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
            "value": 16.46679890852101,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 16.540599609967497,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 28.781640625,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 28.8671875,
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
            "value": 873851.1522268914,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 12960947.190103548,
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
            "value": 23.30024129061321,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 24.808244159851874,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 25.3890625,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 25.48046875,
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
            "value": 335811.4419459192,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 12997961.875793904,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Network Utilization"
          },
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
            "value": 13.669911781098651,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 14.300567241192411,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 23.51171875,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 25.63671875,
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
            "value": 295012.00019319507,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13153972.314145323,
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
            "value": 12.814168040158409,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 13.101841303500429,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 21.408203125,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 21.5,
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
            "value": 539198.3649075365,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13111421.310955489,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - Network Utilization"
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
          "id": "817117a937ed1eff8ddac54beec6417791f83562",
          "message": "Initial API & structure of columnar query engine (#1453)\n\nCloses #1417 & #1409 \n\nDuring last night's SIG meeting for the query/transform engine working\ngroup, there was some consensus that having our Pipelines made up of\nmultiple stages (instead of a single DataFusion `ExecutionPlan`), seemed\nlike a good option. This gives me confidence that the plan laid out in\n#1417 is probably workable - so this PR implements what's spec'd out\nthere with some modifications based on feedback on the issue.\n\nAdds the initial shell of the columnar query engine including the:\n- `Pipeline` - top-level API for invoking an OPL/KQL pipeline on OTAP\nData\n- `PlannedPipeline` - internal data structure used by the `Pipeline`\ncontaining the stages and state needed for adapting to changing OTAP\nbatch schemas\n- `PipelineStage` - trait implemented by a stage in the pipeline to\ntransform OTAP batch\n- `DataFusionPipelineStage` - an implementation of `PipelineStage`\nbacked by a DataFusion `ExecutionPlan`\n- `PipelinePlanner` - used to transform a the\n[`PipelineExpression`](https://github.com/open-telemetry/otel-arrow/blob/a2fc3baf6988b2014a28f12006b56757c82ebc37/rust/experimental/query_engine/expressions/src/pipeline_expression.rs#L7)\ninto `PipelineStages` (not actually implemented)\n- `RecordBatchPartitionStream` - implementation of\n[`PartitionStream`](https://docs.rs/datafusion/latest/datafusion/physical_plan/streaming/trait.PartitionStream.html)\nused to shuffle the current `RecordBatch` for a given OTAP Payload Type\ninto `DataFusionPiplineStage`'s `ExecutionPlan`\n\nThis gives us a concrete base which we can iterate on as we implement\nthe rest of the columnar engine.",
          "timestamp": "2025-11-20T17:48:00Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/817117a937ed1eff8ddac54beec6417791f83562"
        },
        "date": 1763720210771,
        "tool": "customSmallerIsBetter",
        "benches": [
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
            "value": 13.22126767030521,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 14.01690398707394,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 21.075390625,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 21.20703125,
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
            "value": 540475.9918125684,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13137709.215522986,
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
            "value": 16.506239967494068,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 17.046969437924645,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 30.766015625,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 30.84375,
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
            "value": 877829.9825302142,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13042065.368057292,
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
            "value": 22.992826478271137,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 23.62490171058715,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 25.3109375,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 25.41796875,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 125100,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 125100,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 333328.31039903976,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 12943590.764197286,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Network Utilization"
          },
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
            "value": 15.61270964433072,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 16.271977832512317,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 31.243359375,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 31.41796875,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 125100,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 125100,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 293778.79446411546,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13104650.861150395,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - Network Utilization"
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
          "id": "07cf17a583364c4b356724a1fb18cde6c2c0b0d6",
          "message": "Use otelbot app token for Tidy push (#1460)\n\nAnother attempt to get solution working for #1456.\n\nIt *almost* worked as expected in #1458 but I forgot an earlier lesson\nfrom\n[prepare-release](https://github.com/open-telemetry/otel-arrow/blob/382c61d4b35eaf8c4753b046d57aa7bc70d3b0e6/.github/workflows/prepare-release.yml#L133-L134)\nthat we have to inject the special otelbot app token as an environment\nvariable in the task that does the Git action. Otherwise, regular PR\nworkflows are not triggered.\n\nI've seen some evidence that we should opt out of persisting regular git\ncreds from `checkout` in order for the `auth setup-git` to work. Would\nlike to try this configuration.",
          "timestamp": "2025-11-21T23:08:20Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/07cf17a583364c4b356724a1fb18cde6c2c0b0d6"
        },
        "date": 1763806551724,
        "tool": "customSmallerIsBetter",
        "benches": [
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
            "value": 16.53643012143714,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 16.730695388424635,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 28.75,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 28.87890625,
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
            "value": 877515.0401696063,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13023378.284975415,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - Network Utilization"
          },
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
            "value": 14.881939702240604,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 15.092277936360828,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 27.18671875,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 27.3046875,
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
            "value": 299426.83135820366,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13165346.631358014,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - Network Utilization"
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
            "value": 20.762423437773087,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 21.01049443902439,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 26.800390625,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 27.00390625,
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
            "value": 335986.2796613978,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13009183.307346663,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Network Utilization"
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
            "value": 13.307705897798785,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 13.55222039931899,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.255859375,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.33203125,
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
            "value": 541704.732113185,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13185509.740159858,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - Network Utilization"
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
          "id": "07cf17a583364c4b356724a1fb18cde6c2c0b0d6",
          "message": "Use otelbot app token for Tidy push (#1460)\n\nAnother attempt to get solution working for #1456.\n\nIt *almost* worked as expected in #1458 but I forgot an earlier lesson\nfrom\n[prepare-release](https://github.com/open-telemetry/otel-arrow/blob/382c61d4b35eaf8c4753b046d57aa7bc70d3b0e6/.github/workflows/prepare-release.yml#L133-L134)\nthat we have to inject the special otelbot app token as an environment\nvariable in the task that does the Git action. Otherwise, regular PR\nworkflows are not triggered.\n\nI've seen some evidence that we should opt out of persisting regular git\ncreds from `checkout` in order for the `auth setup-git` to work. Would\nlike to try this configuration.",
          "timestamp": "2025-11-21T23:08:20Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/07cf17a583364c4b356724a1fb18cde6c2c0b0d6"
        },
        "date": 1763892943890,
        "tool": "customSmallerIsBetter",
        "benches": [
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
            "value": 12.698316494763922,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 12.927038756412248,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 21.251953125,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 21.3359375,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 125100,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 125100,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 541722.863605879,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13191840.89467844,
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
            "value": 16.369831401605207,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 16.658413161030722,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 25.189453125,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 25.2265625,
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
            "value": 879141.595116541,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13034218.383468702,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - Network Utilization"
          },
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
            "value": 14.4423619391358,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 14.7840655621852,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 25.237109375,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 25.3046875,
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
            "value": 299483.5327374982,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13172475.739303628,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - Network Utilization"
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
            "value": 19.194032820945164,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 19.440793413803775,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.56015625,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.671875,
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
            "value": 335323.07917041273,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 12988480.331936285,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Network Utilization"
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
          "id": "07cf17a583364c4b356724a1fb18cde6c2c0b0d6",
          "message": "Use otelbot app token for Tidy push (#1460)\n\nAnother attempt to get solution working for #1456.\n\nIt *almost* worked as expected in #1458 but I forgot an earlier lesson\nfrom\n[prepare-release](https://github.com/open-telemetry/otel-arrow/blob/382c61d4b35eaf8c4753b046d57aa7bc70d3b0e6/.github/workflows/prepare-release.yml#L133-L134)\nthat we have to inject the special otelbot app token as an environment\nvariable in the task that does the Git action. Otherwise, regular PR\nworkflows are not triggered.\n\nI've seen some evidence that we should opt out of persisting regular git\ncreds from `checkout` in order for the `auth setup-git` to work. Would\nlike to try this configuration.",
          "timestamp": "2025-11-21T23:08:20Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/07cf17a583364c4b356724a1fb18cde6c2c0b0d6"
        },
        "date": 1763979459117,
        "tool": "customSmallerIsBetter",
        "benches": [
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
            "value": 16.872690544251054,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 17.006892869267304,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 27.230078125,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 27.4453125,
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
            "value": 883676.5943701472,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13035019.133724716,
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
            "value": 22.63997655617865,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 23.345543874475688,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 23.361328125,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.52734375,
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
            "value": 335677.6140727118,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13002985.928818887,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Network Utilization"
          },
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
            "value": 15.955245944447682,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 16.560053185047884,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 25.218359375,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 25.37109375,
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
            "value": 294370.7501425534,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13109602.901646879,
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
            "value": 12.42081749147409,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 12.820282091543833,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 21.16953125,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 21.2890625,
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
            "value": 543045.9997655138,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13199110.378934411,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - Network Utilization"
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
          "id": "0dc05d926ef58004a70332e3cb286f1c7825de7d",
          "message": "[otap-dataflow benchmark] filter processor benchmarks + internal telemetry (#1448)\n\nAdd filter processor scenarios to the nightly benchmark suite\nCollect internal metrics inside the filter processor tracking, number of\nsignals before and after the filtering\n\n\n```rust\n/// Pdata-oriented metrics for the OTAP FilterProcessor\n#[metric_set(name = \"filter.processor.pdata.metrics\")]\n#[derive(Debug, Default, Clone)]\npub struct FilterPdataMetrics {\n    /// Number of log signals consumed\n    #[metric(unit = \"{log}\")]\n    pub log_signals_consumed: Counter<u64>,\n    /// Number of span signals consumed\n    #[metric(unit = \"{span}\")]\n    pub span_signals_consumed: Counter<u64>,\n\n    /// Number of log signals sent\n    #[metric(unit = \"{log}\")]\n    pub log_signals_sent: Counter<u64>,\n    /// Number of span signals sent\n    #[metric(unit = \"{span}\")]\n    pub span_signals_sent: Counter<u64>,\n}\n\n```\n\n---------\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2025-11-25T03:30:16Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/0dc05d926ef58004a70332e3cb286f1c7825de7d"
        },
        "date": 1764066298559,
        "tool": "customSmallerIsBetter",
        "benches": [
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
            "value": 21.11677195227887,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 21.49017663439944,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 21.453125,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 21.5234375,
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
            "value": 340007.16338117357,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13006045.099760048,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Network Utilization"
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
            "value": 13.181142133331534,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 13.379552252670694,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 21.08359375,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 21.24609375,
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
            "value": 541593.0029466585,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13193664.644585993,
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
            "value": 16.612662373507963,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 16.836894175960346,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 25.428515625,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 25.57421875,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 125100,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 125100,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 875136.891583642,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 12959060.550047379,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - Network Utilization"
          },
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
            "value": 15.089869378624035,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 15.29829951121111,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 25.348828125,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 25.49609375,
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
            "value": 294266.9269401948,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13101182.799030673,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - Network Utilization"
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
        "date": 1764152511318,
        "tool": "customSmallerIsBetter",
        "benches": [
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
            "value": 17.702148928089652,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 19.305680377621297,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 27.071875,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 27.22265625,
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
            "value": 869439.9445011722,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 12938277.85348376,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - Network Utilization"
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
            "value": 12.73414327088392,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 13.035318510259389,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 15.25078125,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 15.4296875,
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
            "value": 539073.2601301462,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13124996.125945875,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - Network Utilization"
          },
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
            "value": 14.29696609914927,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 14.539771025085166,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 26.971875,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 27.08984375,
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
            "value": 295757.2224066863,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13167818.993518345,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - Network Utilization"
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
            "value": 22.063621096626708,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 24.79125108597986,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 25.37734375,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 25.49609375,
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
            "value": 334355.67443946714,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 12936058.638999496,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Network Utilization"
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
        "date": 1764238881028,
        "tool": "customSmallerIsBetter",
        "benches": [
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
            "value": 12.430633028890995,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 12.615273858548301,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.430078125,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.47265625,
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
            "value": 541346.3157801334,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13177146.857448172,
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
            "value": 16.230473272765174,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 16.503820262401987,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 26.829296875,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 27.125,
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
            "value": 874827.6292681325,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 12957222.789966514,
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
            "value": 20.339614422056133,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 21.046651352481227,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 25.31796875,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 25.40625,
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
            "value": 335453.98520890967,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 12998410.242901172,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Network Utilization"
          },
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
            "value": 16.13202934567198,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 16.448960520244636,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 25.36328125,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 26,
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
            "value": 295404.1627626297,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13157037.289089207,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - Network Utilization"
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
        "date": 1764325272003,
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
            "value": 16.18295579580435,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 16.38471691810842,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 23.5203125,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.6171875,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 125100,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 125100,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 293586.3803644591,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13117835.915807307,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - Network Utilization"
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
            "value": 19.67381770941885,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 20.31719330945336,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 21.65546875,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 21.80078125,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 125100,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 125100,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 334919.33193166676,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13005825.735958777,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Network Utilization"
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
            "value": 13.621747109936958,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 15.19388817843866,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 21.15859375,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 21.2109375,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 125100,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 125100,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 541860.6932696097,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13187952.216458404,
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
            "value": 16.54291453836455,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 18.797548654961712,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 25.20078125,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 25.6015625,
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
            "value": 880538.9669043962,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13027383.673260942,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - Network Utilization"
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
        "date": 1764412046638,
        "tool": "customSmallerIsBetter",
        "benches": [
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
            "value": 12.804923046949282,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 13.033068856014252,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 22.810546875,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 22.87109375,
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
            "value": 542642.4820190984,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13104811.619187169,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - Network Utilization"
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
            "value": 21.559868969116437,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 22.13637512437811,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 23.27265625,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.35546875,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 125100,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 125100,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 335776.376007279,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13004524.442275075,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Network Utilization"
          },
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
            "value": 14.445221338836973,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 15.740596490354697,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 21.44140625,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 21.515625,
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
            "value": 295181.75669352594,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13159885.06896538,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - Network Utilization"
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
            "value": 16.27257655130166,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 16.59549731853117,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 25.301171875,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 25.421875,
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
            "value": 873033.9072760979,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 12942862.868156556,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - Network Utilization"
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
        "date": 1764498055126,
        "tool": "customSmallerIsBetter",
        "benches": [
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
            "value": 15.738193241520726,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 16.50318901404104,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 24.977734375,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 25.0703125,
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
            "value": 874840.9220097356,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 12950502.267673464,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - Network Utilization"
          },
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
            "value": 14.445676748290834,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 15.828736674153824,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 26.978125,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 27.703125,
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
            "value": 294826.74792316236,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13166995.382450823,
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
            "value": 12.308539190945703,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 14.034695338020553,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 21.14765625,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 21.32421875,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 125100,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 125100,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 541106.361073285,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13179929.453871848,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - Network Utilization"
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
            "value": 20.49308846860777,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 20.820347486797143,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 26.875390625,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 26.97265625,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 125100,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 125100,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 339698.45983731106,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 12995218.9940738,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Network Utilization"
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
        "date": 1764584545985,
        "tool": "customSmallerIsBetter",
        "benches": [
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
            "value": 15.975825861314602,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 16.333336048905053,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 22.845703125,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.0234375,
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
            "value": 877981.7214837592,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13003756.907250363,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - Network Utilization"
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
            "value": 12.426267677551065,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 12.716303196347031,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.24296875,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.3671875,
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
            "value": 540744.3001175019,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13173644.3259291,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - Network Utilization"
          },
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
            "value": 14.658760595183173,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 15.091466354967704,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.37734375,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.50390625,
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
            "value": 293700.43443831103,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13102179.97530109,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - Network Utilization"
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
            "value": 20.52480280934667,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 22.163384683025004,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 25.053125,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 25.1484375,
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
            "value": 338855.58909695054,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 12977125.919792552,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Network Utilization"
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
        "date": 1764670904857,
        "tool": "customSmallerIsBetter",
        "benches": [
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
            "value": 21.004818442051345,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 21.515534803392445,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 25.330859375,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 25.51171875,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 125100,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 125100,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 334067.7580229605,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 12926304.83275413,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Network Utilization"
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
            "value": 16.181644299209015,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 16.59018329615861,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 24.707421875,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 24.80859375,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 125100,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 125100,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 877587.4655759751,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13015099.476434527,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - Network Utilization"
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
            "value": 13.246475983270328,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 13.887104703242281,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.180859375,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.296875,
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
            "value": 540827.2012755425,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13188000.527681712,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - Network Utilization"
          },
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
            "value": 15.970840118707802,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 16.92388680626648,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 21.515234375,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 21.58984375,
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
            "value": 293868.1844980561,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13099807.555976892,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - Network Utilization"
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
        "date": 1764757858029,
        "tool": "customSmallerIsBetter",
        "benches": [
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
            "value": 16.187563537060225,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 16.46472641015716,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 23.23125,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.4375,
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
            "value": 872925.1015148464,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 12948287.465395542,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - Network Utilization"
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
            "value": 13.095903007040608,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 13.863257026275116,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.31171875,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.59765625,
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
            "value": 545244.0270879266,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13183363.65797413,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - Network Utilization"
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
            "value": 22.048690632213948,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 22.28665180734087,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 24.9765625,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 25.0859375,
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
            "value": 335336.1121521317,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 12983595.54408226,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Network Utilization"
          },
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
            "value": 15.728578933860538,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 18.468456717386267,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 26.892578125,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 27,
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
            "value": 293890.05815516005,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13092847.73565886,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - Network Utilization"
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
        "date": 1764843718630,
        "tool": "customSmallerIsBetter",
        "benches": [
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
            "value": 20.47635916362814,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 21.205828100855886,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 27.044140625,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 27.16015625,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 125100,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 125100,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 334311.14605491393,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 12937618.567811305,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Network Utilization"
          },
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
            "value": 16.94027364680723,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 19.917851332868107,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 25.275390625,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 25.43359375,
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
            "value": 295999.4220658265,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13180808.940675845,
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
            "value": 11.373931846980794,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 11.753364090662952,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 21.339453125,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 21.44921875,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 125100,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 125100,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 585510.1676680001,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13172553.463231202,
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
            "value": 15.644465351289222,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 16.760880489201952,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 23.023046875,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.1171875,
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
            "value": 1010019.4061548155,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 12971926.234051686,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - Network Utilization"
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
        "date": 1764930516525,
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
            "value": 15.28101519515435,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 16.284085292605003,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 15.28101519515435,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 16.284085292605003,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 21.777083333333334,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 22.0078125,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 325200,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 325200,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 294779.9854921829,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13134739.338489292,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - Network Utilization"
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
            "value": 14.518798099235816,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 16.59995121664475,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 14.518798099235816,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 16.59995121664475,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 25.15234375,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 25.28515625,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 325100,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 325100,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 1010017.2768789482,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 12998344.500560189,
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
            "value": 20.424099067678515,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 21.005556183395292,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 20.424099067678515,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 21.005556183395292,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 26.834765625,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 26.98046875,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 325200,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 325200,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 334912.63511572446,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 12972632.279755553,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Network Utilization"
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
            "value": 12.158743210157606,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 14.341243771950182,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 12.158743210157606,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 14.341243771950182,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 21.129166666666666,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 21.3359375,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 325100,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 325100,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 584491.8239080714,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13148549.111074397,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - Network Utilization"
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
        "date": 1765016858037,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 320200,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 98.46248626708984,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 22.474427230874706,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 23.611288367267942,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 22.474427230874706,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 23.611288367267942,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 29.039973958333334,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 31.7421875,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 325200,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 5000,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 41913.48515948414,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1623798.3396456754,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 320200,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 98.46248626708984,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 15.729180608595373,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 19.081366135784354,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 15.729180608595373,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 19.081366135784354,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 25.001171875,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 25.3046875,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 325200,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 5000,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 126378.92068012322,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1624285.9055576546,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 320100,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 98.4317398071289,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 11.770945002065147,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 12.558392402096825,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 11.770945002065147,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 12.558392402096825,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 22.783723958333333,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.00390625,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 325200,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 5100,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 73087.29667101946,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1645089.6972385156,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 320200,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 98.46248626708984,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 14.070491855896652,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 15.171021728395061,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 14.070491855896652,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 15.171021728395061,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 28.919661458333334,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 29.15234375,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 325200,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 5000,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 36823.85672438758,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1639682.5594074382,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - Network Utilization"
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
        "date": 1765103233628,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 320100,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 98.46200561523438,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 13.077942874184286,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 13.52946726736057,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 13.077942874184286,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 13.52946726736057,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 23.437369791666665,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.58203125,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 325100,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 5000,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 36616.65186153321,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1642505.3560926248,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 320100,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 98.4317398071289,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 20.815931221323776,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 25.618266580046406,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 20.815931221323776,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 25.618266580046406,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 28.948567708333332,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 29.125,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 325200,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 5100,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 41906.996486996904,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1621133.5308415783,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 322700,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 99.23124694824219,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 15.673645021899619,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 22.633808677346952,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 15.673645021899619,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 22.633808677346952,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 28.869140625,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 28.98046875,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 325200,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 2500,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 126330.24293826993,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1624459.1331490846,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 320100,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 98.46200561523438,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 12.377893799577347,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 17.271353657479096,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 12.377893799577347,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 17.271353657479096,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 26.870963541666665,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 26.98046875,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 325100,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 5000,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 73114.55868486194,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1645419.025518517,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - Network Utilization"
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
        "date": 1765189771330,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 320100,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 98.46200561523438,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 20.425177659272105,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 22.673889571008207,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 20.425177659272105,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 22.673889571008207,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 28.977864583333332,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 29.1640625,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 325100,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 5000,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 41898.17884331203,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1620844.8082938395,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 320200,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 98.46248626708984,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 14.593801873082638,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 16.05200196584499,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 14.593801873082638,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 16.05200196584499,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 23.578385416666666,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.734375,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 325200,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 5000,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 37045.71876557178,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1643008.6748128582,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 320200,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 98.46248626708984,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 11.862357706693146,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 12.50862699751861,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 11.862357706693146,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 12.50862699751861,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 22.870833333333334,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 22.92578125,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 325200,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 5000,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 73090.24834050643,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1644913.362823653,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 320200,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 98.46248626708984,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 14.713961183657704,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 15.541159889196676,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 14.713961183657704,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 15.541159889196676,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 27.052994791666666,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 27.234375,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 325200,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 5000,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 124613.28913742161,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1624347.4460535417,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - Network Utilization"
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
        "date": 1765449288056,
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
            "value": 16.14733275258949,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 16.86107625115991,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 16.14733275258949,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 16.86107625115991,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 23.398697916666666,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.54296875,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 325100,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 325100,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 36779.014960937646,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1639799.6443936105,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - Network Utilization"
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
            "value": 20.284707454869974,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 20.75531576048275,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 20.284707454869974,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 20.75531576048275,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 25.423697916666665,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 25.546875,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 325200,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 325200,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 41801.11188311908,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1622341.4018579791,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Network Utilization"
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
            "value": 14.362077800620805,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 14.748902669557138,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 14.362077800620805,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 14.748902669557138,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 21.540625,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 21.9453125,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 325200,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 325200,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 126384.26355437997,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1623852.9550096756,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - Network Utilization"
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
            "value": 11.635474101495333,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 12.804664767801857,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 11.635474101495333,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 12.804664767801857,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 21.209244791666666,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 21.359375,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 325100,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 325100,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 73107.5248431233,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1645306.1992135518,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - Network Utilization"
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
        "date": 1765535682957,
        "tool": "customSmallerIsBetter",
        "benches": [
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
            "value": 21.13336303631284,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 21.59594105002323,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 21.13336303631284,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 21.59594105002323,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 27.395833333333332,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 29.3125,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 325100,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 325100,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 41914.81589299572,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1623569.4797773124,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Network Utilization"
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
            "value": 12.250052621408175,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 12.975556216633235,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 12.250052621408175,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 12.975556216633235,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 23.118098958333334,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.20703125,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 325200,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 325200,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 72913.46804574646,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1642982.3564710543,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - Network Utilization"
          },
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
            "value": 14.819493505614584,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 16.3270651242741,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 14.819493505614584,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 16.3270651242741,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 27.35625,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 27.9765625,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 325200,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 325200,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 36774.89961265407,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1642819.222837247,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - Network Utilization"
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
            "value": 14.832562423776691,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 18.069784779309277,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 14.832562423776691,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 18.069784779309277,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 27.187630208333335,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 27.265625,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 325200,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 325200,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 125987.82765660527,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1623921.8470699838,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - Network Utilization"
          }
        ]
      }
    ]
  }
}