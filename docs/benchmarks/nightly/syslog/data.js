window.BENCHMARK_DATA = {
  "lastUpdate": 1761560077868,
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
      }
    ]
  }
}