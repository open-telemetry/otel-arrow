window.BENCHMARK_DATA = {
  "lastUpdate": 1774114724267,
  "repoUrl": "https://github.com/open-telemetry/otel-arrow",
  "entries": {
    "Benchmark": [
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
        "date": 1774114723437,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.11579055131598,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.25920745920746,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 13.344010416666666,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 14.31640625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 178570.03281600165,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 178570.03281600165,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003349,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 871146.5047422568,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 47699223.96367028,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.17074340382561,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.39005968529571,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.083072916666666,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 36.7421875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 124100.56340485827,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 124100.56340485827,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002145,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 755408.6301261554,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 32657579.53818998,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.99921242279767,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 97.50093585373412,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 32.81432291666667,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 33.37109375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 124548.9180741876,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 124548.9180741876,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002127,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2149197.338559726,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 32728199.021581534,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 90.65088271345235,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 91.1089608177172,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 26.993619791666667,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 27.3203125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 257020.76407183142,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 257020.76407183142,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001767,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2608254.4641262568,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 68816315.881679,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Network Utilization"
          }
        ]
      }
    ]
  }
}