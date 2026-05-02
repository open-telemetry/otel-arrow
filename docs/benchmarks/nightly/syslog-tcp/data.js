window.BENCHMARK_DATA = {
  "lastUpdate": 1777751365988,
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
        "date": 1774144767224,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.15531855565666,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.363155373561,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 15.818489583333333,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 32.1953125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 124240.36583435438,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 124240.36583435438,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002238,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 751824.4207976531,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 32685902.471383777,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 95.83502245155651,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.13947955390336,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 27.5453125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 27.984375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 268745.12809562794,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 268745.12809562794,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002204,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2745467.2518579396,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 71847880.67351599,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.10441013226445,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.26876895834937,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 13.332942708333333,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 13.99609375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 176721.76149697995,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 176721.76149697995,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.007324,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 866084.8055426452,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 47200372.85169265,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.89065975342896,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 97.31439788128992,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 32.203515625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 33.04296875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 124683.03063952996,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 124683.03063952996,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.008166,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2147821.912993537,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 32727963.70195985,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Network Utilization"
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
        "date": 1774200103041,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.1736661376897,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.4019463968487,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 14.380598958333334,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 15.80859375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 125144.02629309507,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 125144.02629309507,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002065,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 756563.445090793,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 32942375.389718685,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 95.64464936843852,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 95.96449128244866,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 27.19921875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 27.7890625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 270144.83382688934,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 270144.83382688934,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.0067,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2731736.2561045024,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 72078880.58919479,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 97.01964520268692,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 97.44232889439455,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 33.00377604166667,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 33.44140625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 125369.01019196522,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 125369.01019196522,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002069,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2163111.980918162,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 32925954.73952328,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.14117026483463,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.26038729666924,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 15.11640625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 16.1875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 178337.47713075636,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 178337.47713075636,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002531,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 872055.5982561925,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 47647175.29008424,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Network Utilization"
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
        "date": 1774233762122,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 12.652338027954102,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 0.2244810490652247,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 0.516738912456614,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 33.598958333333336,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 34.1796875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 7499.667264762354,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 6569.708523931821,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002662,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 33231.68953564648,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 27584.101666523497,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 95.90856689468397,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.49217873651772,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 27.499479166666667,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 28.078125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 268355.5628293738,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 268355.5628293738,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00211,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2739357.1703407783,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 71777557.20646256,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.14832688981055,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.31927076091154,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.029036458333334,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 33.30859375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 124390.44938272674,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 124390.44938272674,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002195,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 757263.3281994106,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 32722884.733493034,
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
            "value": 93.51112760813149,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 97.40209352740786,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 33.199609375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 34.4609375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 121587.42529198105,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 121587.42529198105,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002093,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2085028.5407080424,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 31875781.527436297,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Network Utilization"
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
          "id": "a88939a5ba8b2a740f54e23fd207c32577b6b6dc",
          "message": "fix(deps): Fixes for latest Renovate config (#2413)\n\n# Change Summary\n\n1. Renovate grouping isn't working quite correctly:\n    * #2402\n    * #2403\n    * #2404\n  \nSince these are coming as git refs and not from `crates.io`, I think we\nhave to use the [cargo\nmanager](https://docs.renovatebot.com/modules/manager/cargo/) instead of\n[crate\ndataSource](https://docs.renovatebot.com/modules/datasource/crate/).\n\n2. pip_requirements manager is still trying to update indirect\ndependencies from `requirements.lock.txt` files:\n    * #2401 \n\nLooking at [Renovate job\nlogs](https://developer.mend.io/github/open-telemetry/otel-arrow) - the\nproblem is that while the `pip_compile` correctly skips indirect deps,\n`pip_requirements` was still active on lock files.\n\n## What issue does this PR close?\n\nN/A\n\n## How are these changes tested?\n\nN/A\n\n## Are there any user-facing changes?\n\nNo",
          "timestamp": "2026-03-23T17:54:50Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/a88939a5ba8b2a740f54e23fd207c32577b6b6dc"
        },
        "date": 1774293483453,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 97.08519128737515,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 97.28287462247349,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 33.26940104166667,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 33.7578125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 123862.0837695672,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 123862.0837695672,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00222,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2142726.7201475105,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 32605103.186161317,
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
            "value": 90.26328445303692,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 90.47839059114825,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 26.432161458333333,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 27.04296875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 255821.6128772806,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 255821.6128772806,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002358,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2588727.445312946,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 68590575.67713907,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.1109798003044,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.27192191494308,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 13.997395833333334,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 15.02734375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 178086.91838139796,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 178086.91838139796,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.007215,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 851480.23575351,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 47513174.337407455,
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
            "value": 100.19886015013901,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.3533090571008,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 12.987760416666667,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 13.84765625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 124378.93031920002,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 124378.93031920002,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002124,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 755702.5449755507,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 32675119.455411587,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Network Utilization"
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
          "id": "f1dfdbbe82e8aaa35fb7552e60d2d036fd1dc1db",
          "message": "Use OTAP spec aware `concatenate` when producing the results of `if`/`else` statements (#2393)\n\n# Change Summary\n\n<!--\nReplace with a brief summary of the change in this PR\n-->\n\nIn the columnar query engine, when we write `if`/`else` statements in\nOPL, the results of each branch are concatenated together. Before this\nchange, we were simply using arrow's `concat_batches` helper function\nwith expects all the `RecordBatch`s to have the same schema. However,\nthis would cause a problem if some branch of the statement changes the\nschema.\n\nThis PR corrects the issue by using OTAP's\n[`concatenate`](https://github.com/open-telemetry/otel-arrow/blob/eaa4103326057ef68125244171801bc010cb3571/rust/otap-dataflow/crates/pdata/src/otap/transform/concatenate.rs#L75)\nfunction instead which correctly expands each `RecordBatch` into a\ncommon schema.\n\nThere's one pipeline stage that also writes new IDs to the rows with\nnull IDs (this happens when we assign attributes). In order for\n`concatenate` to produce a valid batch, we need to ensure the IDs are\nglobally unique. This PR adds a mechanism to initialize shared state for\ndifferent implementations of the same pipeline stage if they're being\nused in a nested branch within conditional pipeline stage, and uses it\nfor the purpose of ensuring unique IDs when filling in these null rows.\n\n## What issue does this PR close?\n\n<!--\nWe highly recommend correlation of every PR to an issue\n-->\n\n* Closes #2216 \n\n## How are these changes tested?\n\nUnit tests\n\n## Are there any user-facing changes?\n\nNo\n\n <!-- If yes, provide further info below -->",
          "timestamp": "2026-03-24T01:20:26Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/f1dfdbbe82e8aaa35fb7552e60d2d036fd1dc1db"
        },
        "date": 1774317248435,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.14355542176061,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.34338862191908,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 15.68515625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 30.828125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 124257.33008584667,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 124257.33008584667,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002094,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 753997.063384604,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 32729243.1323209,
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
            "value": 97.25968167557532,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 97.65717029350267,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 33.180598958333334,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 33.8359375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 125144.70830828266,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 125144.70830828266,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001738,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2161407.0213753195,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 32917871.621505734,
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
            "value": 95.91188578074443,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.18794204704783,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 27.238932291666668,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 28.171875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 267907.02201387414,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 267907.02201387414,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00216,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2729950.1220376194,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 71623010.04141346,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.14453727238657,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.34500038618985,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 13.363541666666666,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 14.1171875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 178047.6433222368,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 178047.6433222368,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.005849,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 857235.6027465428,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 47610624.268726826,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Network Utilization"
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
          "id": "3bc95b9d9642d2b00009807923d34006e6502ef5",
          "message": "Remove \"Beaubourg\" Rust prototype (#2414)\n\n# Change Summary\n\nRetires the `rust/beaubourg` prototype from the repo. I believe it has\nserved it's purpose! Adds to the `rust/README.md` a permalink so we can\nfind it easily.\n\nThank you @lquerel. \n\n## What issue does this PR close?\n\nSee https://github.com/open-telemetry/otel-arrow/pull/293\n\n---------\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>\nCo-authored-by: Drew Relmas <drewrelmas@gmail.com>",
          "timestamp": "2026-03-24T16:10:35Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/3bc95b9d9642d2b00009807923d34006e6502ef5"
        },
        "date": 1774373927882,
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
            "value": 100.13301220863246,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.28591503520855,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 14.05,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 14.9375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 177189.55467575186,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 177189.55467575186,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003537,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 860289.1324839923,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 47329700.75081727,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 90.29935840710108,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 90.64418971110769,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 27.145182291666668,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 28.3125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 255997.30645979042,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 255997.30645979042,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.007272,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2591839.0814549862,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 68461596.3064981,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 97.03112740895537,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 97.3582172528923,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 33.049739583333334,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 35.07421875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 124853.90726232088,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 124853.90726232088,
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
            "value": 2158735.512789917,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 32860709.979964502,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.17228460257834,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.34378027079302,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.781380208333335,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 33.41015625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 124241.81611697929,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 124241.81611697929,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.005562,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 752803.7611403752,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 32725307.662310984,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Network Utilization"
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
          "id": "2abee0d4642c6bf2c2be774c5001320451913e74",
          "message": "Pipeline/group/engine policy precedence: prevent against misuse of unresolved policies (#2392)\n\n# Change Summary\n\nLike #2154 but for the other three policy fields. Make all fields Option\ntypes. Adds a ResolvedPolicies type which strips the Options after\nresolving. There was existing resolve code, but it was not used\nconsistently: this was observed for the `telemetry` policy.\n\n## What issue does this PR close?\n\nFixes #2389.\n\n## How are these changes tested?\n\nOne new test. The `configs/internal-telemetry.yaml` configuration is\nmodified to show the problem. Before the fix, no duration metrics. After\nthe fix, duration metrics, as set by the top-level policy.\n\n---------\n\nCo-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>",
          "timestamp": "2026-03-25T01:06:47Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/2abee0d4642c6bf2c2be774c5001320451913e74"
        },
        "date": 1774403971693,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 90.23504701968415,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 90.5256350751499,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 27.780729166666667,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 28.12890625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 259430.69508573625,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 259430.69508573625,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002152,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2628779.8247839315,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 69415249.74293622,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 97.09273532092814,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 97.50426840633737,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 32.634375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 33.55078125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 124905.58458758482,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 124905.58458758482,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002121,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2156843.1867196513,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 32885734.493559256,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 6.93131685256958,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 4.1870996505155835,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.27530684804451,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 38.078125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 40.27734375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 13204.452455371518,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 12307.822968940887,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002488,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 71737.41943394027,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1862446.8989433197,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 9.910065650939941,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 0.24140023608687722,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 0.43464790909793616,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 33.098697916666666,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 34.2890625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 9614.58689324982,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 8681.293660415724,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002578,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 33350.70684698652,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 32902.79904154351,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Network Utilization"
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
          "id": "a44fe949ace10a2c36794537e4652ca89f2dce94",
          "message": "chore(deps): update dependency duckdb to v1.5.1 (#2418)\n\nThis PR contains the following updates:\n\n| Package | Change |\n[Age](https://docs.renovatebot.com/merge-confidence/) |\n[Confidence](https://docs.renovatebot.com/merge-confidence/) |\n|---|---|---|---|\n| [duckdb](https://redirect.github.com/duckdb/duckdb-python)\n([changelog](https://redirect.github.com/duckdb/duckdb-python/releases))\n| `==1.5.0` → `==1.5.1` |\n![age](https://developer.mend.io/api/mc/badges/age/pypi/duckdb/1.5.1?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/pypi/duckdb/1.5.0/1.5.1?slim=true)\n|\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - \"before 8am every weekday\" (UTC),\nAutomerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My42Ni40IiwidXBkYXRlZEluVmVyIjoiNDMuNjYuNCIsInRhcmdldEJyYW5jaCI6Im1haW4iLCJsYWJlbHMiOlsiZGVwZW5kZW5jaWVzIl19-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2026-03-25T15:50:05Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/a44fe949ace10a2c36794537e4652ca89f2dce94"
        },
        "date": 1774460508815,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 19.603824615478516,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 0.2428493902147373,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 0.5629748495602531,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 33.03854166666667,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 34.05078125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 4858.117875805541,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3923.1593412165503,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002661,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 33217.48216278229,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 27815.375549849818,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 95.95113771595479,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.19885997521685,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 26.52109375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 26.7890625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 272235.9405706082,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 272235.9405706082,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002364,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2781034.6411086773,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 72848908.54772492,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 97.09243739974538,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 97.42495429899303,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 32.209765625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 32.80078125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 125752.27162477338,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 125752.27162477338,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002097,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2167912.818817304,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 32992201.093072575,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.15896236372393,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.30831683168317,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 14.47578125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.6015625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 125615.59717331908,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 125615.59717331908,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002103,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 762139.1380590653,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 32996122.196332958,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Network Utilization"
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
          "id": "238a1f78174fa44e3280a2662aa02277f2befa22",
          "message": "Add data sanitization step for transform processor results (#2434)\n\n# Change Summary\n\n<!--\nReplace with a brief summary of the change in this PR\n-->\n\nAdds a \"sanitization\" step which can be performed on the result produced\nby the transform processor. This passes over all the the columns in all\nthe RecordBatch's and removes any values from dictionary columns that\nhave no keys pointing to them.\n\nThe procedure has some performance overhead, so there is an option to\nskip if if, for example the transformation isn't removing sensitive\ndata, or if something further along in the pipeline would remove the\nhidden arrow data (for example, serializing to OTLP in the OTLP\nexporter).\n\n**Why is this necessary?**\nSome of the arrow compute kernels will perform transformations ignoring\ncertain buffers for best performance if the result would still be a\nsemantically correct arrow array. For example when filtering, the arrow\ncompute kernels only filter dictionary key columns without touching the\ndictionaries.\n\nI'm imagining that someday someone will try to use transform processor\nto try to redact sensitive data, but the values from the rows they\ndeleted will still be present in the arrow buffers. If they then\ntransmitted the data using OTAP exporter (which does a simple arrow IPC\nserialization), the \"redacted\" data has escaped.\n\n**Why blindly do this for all transforms on all columns? Can't the\nquery-engine be smarter about this?**\nMaybe - but it's not as simple as it appears.\n\nFor example, consider when we're filtering. If we did something like:\n`logs | where event_name != \"sensitive_event_name\"`, it might be easy to\nthink that `event_name` is the sensitive column so it's the only one\nthat needs sanitizing. But _maybe_ the user actually knows a-priori that\nany log w/ this event name actually has sensitive data in some other\ncolumn.\n\nWhen it comes to the security of, I feel that it's better to be err on\nthe side of caution.\n\nIn the future we could maybe consider a better system where we let the\nuser provide hints about what fields they consider sensitive.\n\n**What's the performance impact?**\n\nThis sanitizing step adds significant overhead. When adding this\nsanitization step into the query engine's benchmarks, I saw anywhere\nbetween 5%-45% performance overhead on top of just executing the\ntransform pipeline w/ no sanitization. The actual overhead depends on\nthe complexity of the transform and size of output.\n\nThis poor performance is another reason why there's an option to skip\nthis step.\n\n**Should sanitize be on by default?**\n\nMy feeling is yes. If someone forgets to configure this or misconfigures\nit, I feel that it's best to fail on the side of worse\nperformance/better security.\n\n**Does this have to happen in the transform processor?**\n\nI feel that this is a reasonable place to do this, but open to\nsuggestions if anyone feels differently.\n\n## What issue does this PR close?\n\n<!--\nWe highly recommend correlation of every PR to an issue\n-->\n\n* Closes #2313\n\n## How are these changes tested?\n\nUnit tests\n\n## Are there any user-facing changes?\n\nThere's a new user facing config field called `skip_sanitize_result`\n\n <!-- If yes, provide further info below -->\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2026-03-26T01:03:23Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/238a1f78174fa44e3280a2662aa02277f2befa22"
        },
        "date": 1774490514573,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 97.06225046931013,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 97.44902856258224,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 32.85559895833333,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 33.375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 124360.5686184049,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 124360.5686184049,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002138,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2148023.911801971,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 32707760.779183764,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.14076713263916,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.30631239440946,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 13.5546875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 14.3359375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 181134.94052254193,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 181134.94052254193,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002228,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 862195.4701888841,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 48267149.7727082,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 95.94892352497934,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.25924302479211,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 27.427994791666666,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 28.09375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 272584.14475418464,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 272584.14475418464,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.006058,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2784220.34782388,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 72937218.12871364,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.18087597662432,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.34271873301763,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.399348958333334,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 35.21484375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 124390.34381873689,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 124390.34381873689,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001442,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 755931.2878157453,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 32689926.881846625,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Network Utilization"
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
          "id": "ff9034401ece7eb6e46fee56e1abd225bf8c9078",
          "message": "chore(deps): update pipeline perf python dependencies (#2544)\n\nThis PR contains the following updates:\n\n| Package | Change |\n[Age](https://docs.renovatebot.com/merge-confidence/) |\n[Confidence](https://docs.renovatebot.com/merge-confidence/) |\n|---|---|---|---|\n| [grpcio](https://redirect.github.com/grpc/grpc) | `==1.78.1` →\n`==1.80.0` |\n![age](https://developer.mend.io/api/mc/badges/age/pypi/grpcio/1.80.0?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/pypi/grpcio/1.78.1/1.80.0?slim=true)\n|\n| [requests](https://redirect.github.com/psf/requests)\n([changelog](https://redirect.github.com/psf/requests/blob/master/HISTORY.md))\n| `==2.33.0` → `==2.33.1` |\n![age](https://developer.mend.io/api/mc/badges/age/pypi/requests/2.33.1?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/pypi/requests/2.33.0/2.33.1?slim=true)\n|\n\n---\n\n### Release Notes\n\n<details>\n<summary>grpc/grpc (grpcio)</summary>\n\n###\n[`v1.80.0`](https://redirect.github.com/grpc/grpc/releases/tag/v1.80.0)\n\n[Compare\nSource](https://redirect.github.com/grpc/grpc/compare/v1.78.1...v1.80.0)\n\nThis is release 1.80.0\n([glimmering](https://redirect.github.com/grpc/grpc/blob/master/doc/g_stands_for.md))\nof gRPC Core.\n\nFor gRPC documentation, see [grpc.io](https://grpc.io/). For previous\nreleases, see\n[Releases](https://redirect.github.com/grpc/grpc/releases).\n\nThis release contains refinements, improvements, and bug fixes, with\nhighlights listed below.\n\n## Core\n\n- \\[ssl] Implement TLS private key signer in Python.\n([#&#8203;41701](https://redirect.github.com/grpc/grpc/pull/41701))\n- \\[TLS Credentials]: Private Key Offload Implementation.\n([#&#8203;41606](https://redirect.github.com/grpc/grpc/pull/41606))\n- Fix max sockaddr struct size on OpenBSD.\n([#&#8203;40454](https://redirect.github.com/grpc/grpc/pull/40454))\n- \\[core] Enable EventEngine for Python by default, and EventEngine fork\nsupport in Python and Ruby.\n([#&#8203;41432](https://redirect.github.com/grpc/grpc/pull/41432))\n- \\[TLS Credentials]: Create InMemoryCertificateProvider to update\ncertificates independently.\n([#&#8203;41484](https://redirect.github.com/grpc/grpc/pull/41484))\n- \\[Ruby] Build/test ruby 4.0 and build native gems with Ruby 4.0\nsupport.\n([#&#8203;41324](https://redirect.github.com/grpc/grpc/pull/41324))\n- \\[EventEngine] Remove an incorrect std::move in DNSServiceResolver\nconstructor.\n([#&#8203;41502](https://redirect.github.com/grpc/grpc/pull/41502))\n- \\[RR and WRR] enable change to connect from a random index.\n([#&#8203;41472](https://redirect.github.com/grpc/grpc/pull/41472))\n- \\[xds] Implement gRFC A101.\n([#&#8203;41051](https://redirect.github.com/grpc/grpc/pull/41051))\n\n## C++\n\n- \\[C++] Add SNI override option to C++ channel credentials options API.\n([#&#8203;41460](https://redirect.github.com/grpc/grpc/pull/41460))\n\n## C\\#\n\n- \\[C# tools] Option to append Async to server side method names\n[#&#8203;39010](https://redirect.github.com/grpc/grpc/issues/39010).\n([#&#8203;39797](https://redirect.github.com/grpc/grpc/pull/39797))\n\n## Objective-C\n\n- \\[Fix]\\[Compiler] Plugins fall back to the edition 2023 for older\nprotobuf.\n([#&#8203;41357](https://redirect.github.com/grpc/grpc/pull/41357))\n\n## PHP\n\n- \\[PHP] Disable php infinite recursion check for callback from Core to\nPHP. ([#&#8203;41835](https://redirect.github.com/grpc/grpc/pull/41835))\n- \\[PHP] Fix runtime error with PHp8.5 alpha because\nzend\\_exception\\_get\\_defaul….\n([#&#8203;40337](https://redirect.github.com/grpc/grpc/pull/40337))\n\n## Python\n\n- \\[Python] Fix `GRPC_TRACE` not working when absl log initialized in\ncython.\n([#&#8203;41814](https://redirect.github.com/grpc/grpc/pull/41814))\n- Revert \"\\[Python] Align GRPC\\_ENABLE\\_FORK\\_SUPPORT env defaults in\ncore and python\n([#&#8203;41455](https://redirect.github.com/grpc/grpc/issues/41455))\".\n([#&#8203;41769](https://redirect.github.com/grpc/grpc/pull/41769))\n- \\[Python] Fix AsyncIO Server maximum\\_concurrent\\_rpcs enforcement\npreventing negative active\\_rpcs count.\n([#&#8203;41532](https://redirect.github.com/grpc/grpc/pull/41532))\n- \\[Python] Docs: correct `grpc.Compression` references.\n([#&#8203;41705](https://redirect.github.com/grpc/grpc/pull/41705))\n- \\[Python] \\[Typeguard] Part 4 - Add Typeguard to AIO stack in tests .\n([#&#8203;40226](https://redirect.github.com/grpc/grpc/pull/40226))\n- \\[Python] Fix multi-thread exception for Asyncio gRPC clients - Fixes\n[#&#8203;25364](https://redirect.github.com/grpc/grpc/issues/25364).\n([#&#8203;41483](https://redirect.github.com/grpc/grpc/pull/41483))\n- \\[Python] Resolve absl::InitializeLog warning.\n([#&#8203;39779](https://redirect.github.com/grpc/grpc/pull/39779))\n- \\[Python] Remove IF usage in Cython.\n([#&#8203;41400](https://redirect.github.com/grpc/grpc/pull/41400))\n- \\[Python] Add language features to exported proto files.\n([#&#8203;41501](https://redirect.github.com/grpc/grpc/pull/41501))\n- \\[Python] Fix crash when iterating on AIO Metadata keys(), values(),\nitems() or list(metadata.values()) etc. .\n([#&#8203;41481](https://redirect.github.com/grpc/grpc/pull/41481))\n- \\[Python] Modernize and revamp Public API Docs.\n([#&#8203;41287](https://redirect.github.com/grpc/grpc/pull/41287))\n\n## Ruby\n\n- \\[Ruby] Added support to push native-debug packages off rubygems to\npublic gcs bucket.\n([#&#8203;41270](https://redirect.github.com/grpc/grpc/pull/41270))\n\n</details>\n\n<details>\n<summary>psf/requests (requests)</summary>\n\n###\n[`v2.33.1`](https://redirect.github.com/psf/requests/blob/HEAD/HISTORY.md#2331-2026-03-30)\n\n[Compare\nSource](https://redirect.github.com/psf/requests/compare/v2.33.0...v2.33.1)\n\n**Bugfixes**\n\n- Fixed test cleanup for CVE-2026-25645 to avoid leaving unnecessary\nfiles in the tmp directory.\n([#&#8203;7305](https://redirect.github.com/psf/requests/issues/7305))\n- Fixed Content-Type header parsing for malformed values.\n([#&#8203;7309](https://redirect.github.com/psf/requests/issues/7309))\n- Improved error consistency for malformed header values.\n([#&#8203;7308](https://redirect.github.com/psf/requests/issues/7308))\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - \"before 8am on Monday\" (UTC),\nAutomerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n👻 **Immortal**: This PR will be recreated if closed unmerged. Get\n[config\nhelp](https://redirect.github.com/renovatebot/renovate/discussions) if\nthat's undesired.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My4xMDIuMTEiLCJ1cGRhdGVkSW5WZXIiOiI0My4xMDIuMTEiLCJ0YXJnZXRCcmFuY2giOiJtYWluIiwibGFiZWxzIjpbImRlcGVuZGVuY2llcyJdfQ==-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2026-04-06T15:32:43Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/ff9034401ece7eb6e46fee56e1abd225bf8c9078"
        },
        "date": 1775494193699,
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
            "value": 100.15782398458623,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.41083894957006,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 15.849869791666666,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 30.48046875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 176055.41083106847,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 176055.41083106847,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002132,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 885318.4359698301,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 47230976.217682615,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 97.07223796950704,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 97.5626939154668,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 33.37682291666667,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 33.78515625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 122899.83001381742,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 122899.83001381742,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002524,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2129178.3175116344,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 32285274.188455388,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.16031599982573,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.30759122956536,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 14.004036458333333,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 14.91796875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 123034.40443468504,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 123034.40443468504,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001916,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 762938.3008938166,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 32356845.382894907,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 94.1266808735523,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.22640737584257,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 28.483333333333334,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 29.21875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 262930.5447046257,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 262930.5447046257,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002538,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2686450.2648012335,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 70363127.45670359,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Network Utilization"
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
          "id": "962ac30ac5ba928f035dbc513a34ed75787d83c4",
          "message": "[otap-dataflow] add transport header infrastructure (#2539)\n\n# Change Summary\n\nUpdate the OtapPdata Context with the transport_headers field, so that\nOtapPdata can carry the headers through the pipeline.\n\nDefined the TransportHeadersPolicy with HeaderCapturePolicy and\nHeaderPropagationPolicy, update the Policies to have a transport_headers\nfield. Receiver and Exporter nodes can also define HeaderCapturePolicy\nand HeaderPropagationPolicy respectively, these definitions will\noverride any top level HeaderCapturePolicy and HeaderPropagationPolicy\nrules.\n\nExposed the policy to the Receiver and Exporter nodes via the\nEffectHandler with helper functions that apply the policies on a\niterator of key value pairs (for receiver nodes) and transport_headers\n(for exporter nodes)\n\n## What issue does this PR close?\n\n* Closes #2508\n\n## How are these changes tested?\n\nunit tests and integration tests\n\n## Are there any user-facing changes?\n\nno",
          "timestamp": "2026-04-07T00:18:20Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/962ac30ac5ba928f035dbc513a34ed75787d83c4"
        },
        "date": 1775524966903,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 6.980602264404297,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 3.9579046078635556,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.21711806631546,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 38.20078125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 40.34375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 13039.557089710854,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 12147.920708959919,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002038,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 80259.57746124957,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1744049.1555150999,
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
            "value": 97.01457051982322,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 97.21352836467766,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 32.36744791666667,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 33.39453125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 122775.21381291487,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 122775.21381291487,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002339,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2126873.083991484,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 32217729.84711268,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.11074552696266,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.23920253458,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 15.798307291666667,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 25.8515625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 177107.8596014604,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 177107.8596014604,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002419,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 855284.2994660791,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 47635731.36866291,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 95.73026223757275,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.19820377825953,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 27.251171875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 28.2734375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 266633.75229383423,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 266633.75229383423,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002156,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2707737.1739269015,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 71280165.8751782,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Network Utilization"
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
          "id": "2bc828b31d6c8aadb0c03b256d8c2c762f63af18",
          "message": "fix(query_engine): remove cross-workspace path dep that broke external git consumers (#2567)\n\n## Summary\n\nFixes #2560\n\n- Remove the `otap-df-pdata` dev-dependency from\n`engine-recordset-otlp-bridge` that created a circular cross-workspace\nreference between `query_engine` and `otap-dataflow`\n- Inline the benchmark fixture using `opentelemetry-proto` types already\navailable as dev-dependencies\n\n## Root Cause\n\nPR #2453 added `data_engine_recordset_otlp_bridge` as a workspace\ndependency in `otap-dataflow`, which meant `otap-dataflow` now\nreferenced crates in the `query_engine` workspace. Meanwhile,\n`engine-recordset-otlp-bridge` (a `query_engine` member) had a\ndev-dependency pointing back into `otap-dataflow`:\n\n```toml\n# rust/experimental/query_engine/engine-recordset-otlp-bridge/Cargo.toml\n[dev-dependencies]\notap-df-pdata = { path = \"../../../otap-dataflow/crates/pdata\", features = [\"testing\"] }\n```\n\nThis bidirectional cross-workspace reference causes cargo's package\ndiscovery to fail when resolving any `otap-dataflow` crate as an\nexternal git dependency:\n\n```\nerror: no matching package named `otap-df-pdata` found\nlocation searched: Git repository .../otel-arrow.git?rev=...\n```\n\n## Fix\n\n**`engine-recordset-otlp-bridge/Cargo.toml`** — Removed the\n`otap-df-pdata` dev-dependency.\n\n**`engine-recordset-otlp-bridge/benches/extend.rs`** — Replaced the\nimported `logs_with_varying_attributes_and_properties` fixture with a\nself-contained `generate_logs_batch` function that builds equivalent\n`ExportLogsServiceRequest` protobuf bytes using `opentelemetry-proto`\ntypes (already a dev-dependency). The benchmark logic is otherwise\nunchanged.\n\n## Validation\n\n- `cargo check --workspace` in `query_engine` — passes\n- `cargo bench -p data_engine_recordset_otlp_bridge --no-run` —\nbenchmark compiles\n- `cargo xtask check` in `otap-dataflow` — structure checks, fmt, clippy\n(0 warnings), all tests pass\n- Tested as external git dep consumer from a downstream project — cargo\nresolves all `otap-df-*` packages successfully",
          "timestamp": "2026-04-07T16:29:18Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/2bc828b31d6c8aadb0c03b256d8c2c762f63af18"
        },
        "date": 1775585044635,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.5363035202026367,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 36.56566769434962,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.37684455115792,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 32.085026041666666,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 40.58984375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 49888.12254365494,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 49134.81755969917,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002258,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 307503.6797385539,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 12087236.626021277,
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
            "value": 97.03974770982367,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 97.30251181529403,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 33.847265625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 34.9609375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 124593.59254713691,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 124593.59254713691,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002283,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2160126.641119967,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 32721103.01339333,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.14884191480957,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.30676058518462,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 15.64453125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 26.578125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 178962.59691390765,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 178962.59691390765,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002482,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 865551.6840949396,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 47877283.74728734,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 95.63684033085694,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.13820051413882,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 26.86796875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 27.36328125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 270204.86933392886,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 270204.86933392886,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.0041,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2753624.221840931,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 72121849.7572527,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Network Utilization"
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
          "id": "c225a4bfb556577abf133dc52bcf01eb1e901e70",
          "message": "[otap-dataflow] update transport headers struct to wrap headers in arc (#2584)\n\n# Change Summary\n\nUpdated the TransportHeaders struct to have a Arc<Vec<TransportHeader>>\ninstead of Vec<TransportHeader>\n\n## What issue does this PR close?\n\n* Closes #2583\n\n## How are these changes tested?\n\nunit tests\n\n## Are there any user-facing changes?\n\nno",
          "timestamp": "2026-04-07T23:42:31Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/c225a4bfb556577abf133dc52bcf01eb1e901e70"
        },
        "date": 1775612556050,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 95.4010916363521,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.2902672467371,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 27.680729166666666,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 28.33984375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 263636.3584850499,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 263636.3584850499,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002346,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2689274.600702005,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 70607204.1149942,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.18262300244616,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.56271028761921,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.620182291666666,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 35.49609375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 122252.10666308814,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 122252.10666308814,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002238,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 752278.9585873521,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 32091323.29326437,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 15.303071022033691,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 0.2667857678310105,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 0.5957857527088296,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 32.22473958333333,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 33.109375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 6109.7359575777,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5193.108904476845,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002593,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 44420.20420695937,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 27646.904205160045,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 97.14654196259063,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 97.42232140089881,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 33.20846354166667,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 33.70703125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 121985.22962177997,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 121985.22962177997,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.007265,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2109622.0822436265,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 32023738.289626665,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Network Utilization"
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
          "id": "68e472cef0583ced74c6db564ada17c5f8ba7005",
          "message": "fix(otlp-receiver): make request_bytes metric consistent across HTTP and gRPC (#2596)\n\nThe `request_bytes` counter was recording compressed wire size on the\nHTTP path but decompressed size on the gRPC path (where tonic\ndecompresses before handing bytes to the decoder). Move the HTTP\nrecording to after decompression so both protocols report decompressed\npayload bytes for successfully received requests.\n\n**Changes:**\n\n- **`otlp_http.rs`**: Move `request_bytes.add()` from before\n`decode_content_encoding()` to after it, so it records decompressed\npayload size — matching the gRPC path.\n- **`otlp_metrics.rs`**: Update doc comment to clarify the metric\nmeasures decompressed bytes for successfully received requests.",
          "timestamp": "2026-04-08T17:23:23Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/68e472cef0583ced74c6db564ada17c5f8ba7005"
        },
        "date": 1775673339989,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.3299113512039185,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 42.1605727803873,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.41266822066821,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 29.027994791666668,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 39.69140625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 57131.115693859145,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 56384.47800917861,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002329,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 352841.6382879641,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 14135284.651545037,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 9.651202201843262,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 0.26984710194708483,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 0.42936521403779404,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 32.876171875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 34.19140625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 9821.272178900814,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 8892.97613212536,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00241,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 47130.25635571091,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 195455.19795214318,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 90.30328444493277,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 90.62471380992952,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 27.563020833333333,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 27.99609375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 264179.8179372018,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 264179.8179372018,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001934,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2689177.294558989,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 70822109.95402157,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.86530661476934,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 97.2032573591903,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 33.73606770833333,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 34.3203125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 124993.58366270532,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 124993.58366270532,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00308,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2163235.1518276,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 32806386.070458494,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Sameer J",
            "username": "sjmsft",
            "email": "101909410+sjmsft@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "9b4b8dc2c00fb378ee2a7640a1b20189558f7b7f",
          "message": "fix: handle DrainIngress in fake_data_generator to unblock graceful shutdown (#2515)\n\n# Change Summary\n\nThe \"Ack nack redesign\" PR (3dca2837) introduced a two-phase\nDrainIngress/ReceiverDrained shutdown protocol but missed updating the\nfake_data_generator receiver. Without the DrainIngress handler, the\nmessage falls into the _ => {} catch-all, notify_receiver_drained() is\nnever called, the pipeline controller never removes the receiver from\nits pending set, and after the deadline expires it emits\nDrainDeadlineReached. This was causing pipeline-perf-test-basic to fail\nconsistently.\n\n## What issue does this PR close?\n\npipeline-perf-test-basic unit test is failing.\n\n* Closes #2511\n\n## How are these changes tested?\n\nfake_data_generator and runtime_control_metrics tests were executed.\n\n## Are there any user-facing changes?\n\nNo, fake_data_generator is an internal test/load-generation receiver,\nnot a user-facing component.\n\n---------\n\nCo-authored-by: Lalit Kumar Bhasin <lalit_fin@yahoo.com>\nCo-authored-by: Joshua MacDonald <josh.macdonald@gmail.com>\nCo-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>\nCo-authored-by: Joshua MacDonald <jmacd@users.noreply.github.com>",
          "timestamp": "2026-04-09T00:04:43Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/9b4b8dc2c00fb378ee2a7640a1b20189558f7b7f"
        },
        "date": 1775699575456,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 95.60335555370253,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.11853503677894,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 29.41015625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 31.03515625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 270807.75377708575,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 270807.75377708575,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002344,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2774735.1169182444,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 72327520.17192219,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 99.96518991553067,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.39385475728156,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.611328125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 39.671875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 123607.06642367794,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 123607.06642367794,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002233,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 764940.7999952727,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 32524923.9035041,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.1364680129555,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.27314576427966,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.196614583333332,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 16.9609375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 179149.70670056136,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 179149.70670056136,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002331,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 874429.2553378883,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 47766220.021285996,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 97.06677906145558,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 97.4751533932445,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 36.01328125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 36.55078125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 123306.62528560714,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 123306.62528560714,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00813,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2145516.4574933914,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 32418711.78050689,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Network Utilization"
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
          "id": "b5f0814099566c119a29aa8465a137e04adbeeb4",
          "message": "OPL/Columnar Query Engine Support some `TextExpression` variants (#2586)\n\n# Change Summary\n\n<!--\nReplace with a brief summary of the change in this PR\n-->\n\nOPL / Columnar Query Engine support the `Concat`, `Join` and `Replace`\nvariants of the\n[`TextExpression`](https://github.com/open-telemetry/otel-arrow/blob/72fba8d2a94cd5e20403875e9214756a86cd405f/rust/experimental/query_engine/expressions/src/scalars/text_scalar_expression.rs#L7-L23).\nIn all these cases, we parse from specially named functions:\n```js\nlogs | set attributes[\"x\"] = concat(\"the\", \" attribute value \", \"is: \", attributes[\"x\"])\nlogs | set event_name = join(\" \", \"event happened:\", event_name)\nlogs | set event_name = replace(event_name, \"otel\", \"otap\")\n```\n\nIn each of these cases, we use the equivalent datafusion scalar function\n`concat`, `concat_ws` (for join) and `replace`.\n\nNote: `concat_ws` is also used as an alias for `join`. I was thinking\nthis'd be helpful for folks coming from a datafusion/SQL background. So\nit's equally possibly to write an expression like:\n```js\nlogs | set event_name = concat_ws(\" \", \"event happened:\", event_name)\n```\n\nIn `planner.rs`, I refactored the planning of function arguments into a\nreusable helper function.\n\n## What issue does this PR close?\n\n<!--\nWe highly recommend correlation of every PR to an issue\n-->\n\n* Related to #2578\n\n## How are these changes tested?\n\nUnit\n\n## Are there any user-facing changes?\n\n <!-- If yes, provide further info below -->\n \nThese new expression types are now supported via the transform\nprocessor.\n\n## Future work\n\nWill add support for the `TextExpression::Capture` variant of this\nexpression in future PR.",
          "timestamp": "2026-04-09T16:15:29Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/b5f0814099566c119a29aa8465a137e04adbeeb4"
        },
        "date": 1775764192972,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 3.2592408657073975,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 16.650779044438604,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.31201864801864,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 41.27122395833333,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 44.359375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 26673.56851848951,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 25821.94755548119,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00322,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 168726.8872878285,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5580275.12760799,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 22.230981826782227,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 0.2756828400436733,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 0.5913560371517028,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 34.77903645833333,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 35.75390625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 4206.499037680015,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3288.202298471739,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002391,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 46569.301397397874,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 27446.35924337265,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 97.05697563242609,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 97.61714373938551,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 35.430729166666666,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 35.94921875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 124774.72919172361,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 124774.72919172361,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003336,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2166113.0867603845,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 32803475.87281776,
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
            "value": 95.81482156669111,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.0772585669782,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 30.607552083333335,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 31.1484375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 270294.3729262378,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 270294.3729262378,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002359,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2780429.6788056595,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 72442039.24731539,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Network Utilization"
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
          "id": "b5f0814099566c119a29aa8465a137e04adbeeb4",
          "message": "OPL/Columnar Query Engine Support some `TextExpression` variants (#2586)\n\n# Change Summary\n\n<!--\nReplace with a brief summary of the change in this PR\n-->\n\nOPL / Columnar Query Engine support the `Concat`, `Join` and `Replace`\nvariants of the\n[`TextExpression`](https://github.com/open-telemetry/otel-arrow/blob/72fba8d2a94cd5e20403875e9214756a86cd405f/rust/experimental/query_engine/expressions/src/scalars/text_scalar_expression.rs#L7-L23).\nIn all these cases, we parse from specially named functions:\n```js\nlogs | set attributes[\"x\"] = concat(\"the\", \" attribute value \", \"is: \", attributes[\"x\"])\nlogs | set event_name = join(\" \", \"event happened:\", event_name)\nlogs | set event_name = replace(event_name, \"otel\", \"otap\")\n```\n\nIn each of these cases, we use the equivalent datafusion scalar function\n`concat`, `concat_ws` (for join) and `replace`.\n\nNote: `concat_ws` is also used as an alias for `join`. I was thinking\nthis'd be helpful for folks coming from a datafusion/SQL background. So\nit's equally possibly to write an expression like:\n```js\nlogs | set event_name = concat_ws(\" \", \"event happened:\", event_name)\n```\n\nIn `planner.rs`, I refactored the planning of function arguments into a\nreusable helper function.\n\n## What issue does this PR close?\n\n<!--\nWe highly recommend correlation of every PR to an issue\n-->\n\n* Related to #2578\n\n## How are these changes tested?\n\nUnit\n\n## Are there any user-facing changes?\n\n <!-- If yes, provide further info below -->\n \nThese new expression types are now supported via the transform\nprocessor.\n\n## Future work\n\nWill add support for the `TextExpression::Capture` variant of this\nexpression in future PR.",
          "timestamp": "2026-04-09T16:15:29Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/b5f0814099566c119a29aa8465a137e04adbeeb4"
        },
        "date": 1775790492002,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 95.73680823828951,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.15939436075705,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 30.304427083333334,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 30.76171875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 270465.9045364887,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 270465.9045364887,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001648,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2762558.486890088,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 72507189.48078762,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 97.06150426153992,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 97.4031669910471,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 34.73216145833333,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 35.17578125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 124082.26174637467,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 124082.26174637467,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00213,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2155606.7909737877,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 32619788.225740615,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.18110666682401,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.38493682588599,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.788411458333332,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.23046875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 123695.33050127358,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 123695.33050127358,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002265,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 766318.4139560566,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 32570936.70460056,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.18435840961683,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.3318119141684,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.423567708333334,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.2578125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 178676.17735243036,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 178676.17735243036,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002403,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 865116.1550931392,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 47730351.54843029,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Network Utilization"
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
          "id": "b5f0814099566c119a29aa8465a137e04adbeeb4",
          "message": "OPL/Columnar Query Engine Support some `TextExpression` variants (#2586)\n\n# Change Summary\n\n<!--\nReplace with a brief summary of the change in this PR\n-->\n\nOPL / Columnar Query Engine support the `Concat`, `Join` and `Replace`\nvariants of the\n[`TextExpression`](https://github.com/open-telemetry/otel-arrow/blob/72fba8d2a94cd5e20403875e9214756a86cd405f/rust/experimental/query_engine/expressions/src/scalars/text_scalar_expression.rs#L7-L23).\nIn all these cases, we parse from specially named functions:\n```js\nlogs | set attributes[\"x\"] = concat(\"the\", \" attribute value \", \"is: \", attributes[\"x\"])\nlogs | set event_name = join(\" \", \"event happened:\", event_name)\nlogs | set event_name = replace(event_name, \"otel\", \"otap\")\n```\n\nIn each of these cases, we use the equivalent datafusion scalar function\n`concat`, `concat_ws` (for join) and `replace`.\n\nNote: `concat_ws` is also used as an alias for `join`. I was thinking\nthis'd be helpful for folks coming from a datafusion/SQL background. So\nit's equally possibly to write an expression like:\n```js\nlogs | set event_name = concat_ws(\" \", \"event happened:\", event_name)\n```\n\nIn `planner.rs`, I refactored the planning of function arguments into a\nreusable helper function.\n\n## What issue does this PR close?\n\n<!--\nWe highly recommend correlation of every PR to an issue\n-->\n\n* Related to #2578\n\n## How are these changes tested?\n\nUnit\n\n## Are there any user-facing changes?\n\n <!-- If yes, provide further info below -->\n \nThese new expression types are now supported via the transform\nprocessor.\n\n## Future work\n\nWill add support for the `TextExpression::Capture` variant of this\nexpression in future PR.",
          "timestamp": "2026-04-09T16:15:29Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/b5f0814099566c119a29aa8465a137e04adbeeb4"
        },
        "date": 1775842822943,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 97.03971043626252,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 97.29480400527828,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 36.01640625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 36.65625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 124225.91713924496,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 124225.91713924496,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001167,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2160126.1366365906,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 32679630.387674768,
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
            "value": 90.47041766860434,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 90.74081784386617,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 30.280338541666666,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 30.71875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 263411.1617117516,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 263411.1617117516,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004291,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2681056.732762339,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 70515144.59107803,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.16920900513966,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.27908873272067,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.931119791666667,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.5546875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 179607.7857539389,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 179607.7857539389,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00241,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 870418.7890101145,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 48030014.07102462,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 2.468954086303711,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 23.16255784813649,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.31553626782393,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 38.83385416666667,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 43.96875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 33936.85821122975,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 33115.22725812184,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002608,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 212404.4906059475,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 7666890.974393782,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Sameer J",
            "username": "sjmsft",
            "email": "101909410+sjmsft@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "9f8e589b22501ac94dc6a1e89d54da10e2eeeb7e",
          "message": "refactor(engine): extract IndexedMinHeap from node-local wakeup scheduler (#2627)\n\n# Change Summary\n\nExtracts the hand-rolled binary heap (Vec<ScheduledWakeup> +\nHashMap<WakeupSlot, usize>) from NodeLocalScheduler into a generic,\nreusable IndexedMinHeap<K, P> data structure with its own module and\ntest suite.\n\n## What issue does this PR close?\n\n* Closes #2587 \n\n## How are these changes tested?\n\ncargo test -p otap-df-engine -- \"indexed_min_heap|node_local_scheduler\"\n\nExisting unit tests in node_local_scheduler and new unit tests in\nindexed_min_heap.\n\n## Are there any user-facing changes?\n\nNo. All changes are internal to otap-df-engine. No downstream crate or\nuser-facing behavior changes.",
          "timestamp": "2026-04-11T01:08:20Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/9f8e589b22501ac94dc6a1e89d54da10e2eeeb7e"
        },
        "date": 1775873931637,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 93.85175411235895,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 97.19783817266745,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 34.744791666666664,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 35.859375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 120540.51747929164,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 120540.51747929164,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00472,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2078852.2454620956,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 31636735.705158792,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.21285000058427,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.47195248013577,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.034765625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.83203125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 123478.62262388023,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 123478.62262388023,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002289,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 768174.2449257061,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 32509797.01524008,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 95.98153289662685,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.27290097087379,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 30.140885416666666,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 31.34375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 267012.9764863738,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 267012.9764863738,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.008694,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2740178.358334515,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 71418704.93028648,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.16547949173808,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.36648757451421,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.991015625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.46875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 177189.9393553025,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 177189.9393553025,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002278,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 875880.65519884,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 47281717.66318648,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Sameer J",
            "username": "sjmsft",
            "email": "101909410+sjmsft@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "9f8e589b22501ac94dc6a1e89d54da10e2eeeb7e",
          "message": "refactor(engine): extract IndexedMinHeap from node-local wakeup scheduler (#2627)\n\n# Change Summary\n\nExtracts the hand-rolled binary heap (Vec<ScheduledWakeup> +\nHashMap<WakeupSlot, usize>) from NodeLocalScheduler into a generic,\nreusable IndexedMinHeap<K, P> data structure with its own module and\ntest suite.\n\n## What issue does this PR close?\n\n* Closes #2587 \n\n## How are these changes tested?\n\ncargo test -p otap-df-engine -- \"indexed_min_heap|node_local_scheduler\"\n\nExisting unit tests in node_local_scheduler and new unit tests in\nindexed_min_heap.\n\n## Are there any user-facing changes?\n\nNo. All changes are internal to otap-df-engine. No downstream crate or\nuser-facing behavior changes.",
          "timestamp": "2026-04-11T01:08:20Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/9f8e589b22501ac94dc6a1e89d54da10e2eeeb7e"
        },
        "date": 1775930345014,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 95.92326016089439,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.25159680793368,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 29.381901041666666,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 30.34375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 268344.445118492,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 268344.445118492,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00236,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2752923.4064407693,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 71811724.52923928,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.15433959643632,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.39448510638297,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.171744791666665,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.83203125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 178026.743376716,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 178026.743376716,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002221,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 879983.7433913914,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 47530653.22733303,
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
            "value": 100.19559122338353,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.44430080795526,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.61953125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.19140625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 123925.64814432267,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 123925.64814432267,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002107,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 766321.0843283163,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 32621186.238564275,
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
            "value": 97.07299612764643,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 97.58642304118922,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 35.20390625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 35.8828125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 123560.31500472274,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 123560.31500472274,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002275,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2147756.2184140305,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 32462996.892198954,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Sameer J",
            "username": "sjmsft",
            "email": "101909410+sjmsft@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "9f8e589b22501ac94dc6a1e89d54da10e2eeeb7e",
          "message": "refactor(engine): extract IndexedMinHeap from node-local wakeup scheduler (#2627)\n\n# Change Summary\n\nExtracts the hand-rolled binary heap (Vec<ScheduledWakeup> +\nHashMap<WakeupSlot, usize>) from NodeLocalScheduler into a generic,\nreusable IndexedMinHeap<K, P> data structure with its own module and\ntest suite.\n\n## What issue does this PR close?\n\n* Closes #2587 \n\n## How are these changes tested?\n\ncargo test -p otap-df-engine -- \"indexed_min_heap|node_local_scheduler\"\n\nExisting unit tests in node_local_scheduler and new unit tests in\nindexed_min_heap.\n\n## Are there any user-facing changes?\n\nNo. All changes are internal to otap-df-engine. No downstream crate or\nuser-facing behavior changes.",
          "timestamp": "2026-04-11T01:08:20Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/9f8e589b22501ac94dc6a1e89d54da10e2eeeb7e"
        },
        "date": 1775959743755,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 97.12181788657497,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 97.42247560030984,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 34.572916666666664,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 35.17578125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 123784.2384347169,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 123784.2384347169,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.006832,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2145324.474419963,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 32496716.974948917,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.14061748020681,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.33466491051368,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.965494791666668,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.828125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 178411.16715145277,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 178411.16715145277,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00241,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 880453.4991917807,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 47684647.6572408,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 90.57480504163193,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 90.8509833655706,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 29.696223958333334,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 30.16796875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 258144.1950288293,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 258144.1950288293,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002124,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2622319.685524895,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 68948905.62669049,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.18887923786035,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.38107068366165,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.031119791666665,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 16.6953125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 123774.22402909557,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 123774.22402909557,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001992,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 766777.0181412747,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 32578819.65780858,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Network Utilization"
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
          "id": "5868ff15d7307129796bf35ba0f322a08a8d3586",
          "message": " feat: Remove experimental-tls feature flag and make TLS always available  (#2624)\n\n### Summary                                 \n                                          \nRemove the `experimental-tls` feature gate and make TLS support\navailable by\ndefault across OTAP Dataflow.\n   \n### What changed\n                  \n- Removed the `experimental-tls` feature wiring from the workspace and\nall\n    affected crates\n  - Made the core TLS dependencies in `otap-df-otap` unconditional\n  - Removed feature-gated TLS fallback paths and the obsolete\n    `TlsFeatureDisabled` error variant    \n  - Made existing TLS tests compile and run by default\n- Updated configs, scripts, and docs to stop referring to\n`experimental-tls`\n- Added a binary-level compile-time guard in `df_engine` so normal\nbuilds must\nenable exactly one crypto provider:\n    - `crypto-ring`\n- `crypto-aws-lc`\n    - `crypto-openssl`                        \n  ### Notes       \n- This change does not alter the existing `crypto-*` feature flags; it\nonly\nremoves the compile-time gate around TLS availability.\n- `tonic/tls-native-roots` was intentionally not made unconditional.\nNative\ntrust anchors are loaded directly via `rustls_native_certs` in the TLS\nhelper\n    paths, so this is not an omission.",
          "timestamp": "2026-04-12T06:29:36Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/5868ff15d7307129796bf35ba0f322a08a8d3586"
        },
        "date": 1776014930138,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 90.33923450608353,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 90.57444969405934,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 30.374348958333332,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 30.87109375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 260623.88886890994,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 260623.88886890994,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.007546,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2666301.837654947,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 69948249.45079704,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.20032276940502,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.41915562403699,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.426822916666666,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.5625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 123050.63785488805,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 123050.63785488805,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002127,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 762307.3794425587,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 32370237.53247621,
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
            "value": 97.08237822268491,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 97.30945378476795,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 35.125651041666664,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 35.6953125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 124656.16932959923,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 124656.16932959923,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002646,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2161425.7604259523,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 32728898.75966785,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.1349811202805,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.32902564102564,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.275260416666665,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.43359375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 179428.00529826013,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 179428.00529826013,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002339,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 889512.0216894298,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 48046693.20405248,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Network Utilization"
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
          "id": "5868ff15d7307129796bf35ba0f322a08a8d3586",
          "message": " feat: Remove experimental-tls feature flag and make TLS always available  (#2624)\n\n### Summary                                 \n                                          \nRemove the `experimental-tls` feature gate and make TLS support\navailable by\ndefault across OTAP Dataflow.\n   \n### What changed\n                  \n- Removed the `experimental-tls` feature wiring from the workspace and\nall\n    affected crates\n  - Made the core TLS dependencies in `otap-df-otap` unconditional\n  - Removed feature-gated TLS fallback paths and the obsolete\n    `TlsFeatureDisabled` error variant    \n  - Made existing TLS tests compile and run by default\n- Updated configs, scripts, and docs to stop referring to\n`experimental-tls`\n- Added a binary-level compile-time guard in `df_engine` so normal\nbuilds must\nenable exactly one crypto provider:\n    - `crypto-ring`\n- `crypto-aws-lc`\n    - `crypto-openssl`                        \n  ### Notes       \n- This change does not alter the existing `crypto-*` feature flags; it\nonly\nremoves the compile-time gate around TLS availability.\n- `tonic/tls-native-roots` was intentionally not made unconditional.\nNative\ntrust anchors are loaded directly via `rustls_native_certs` in the TLS\nhelper\n    paths, so this is not an omission.",
          "timestamp": "2026-04-12T06:29:36Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/5868ff15d7307129796bf35ba0f322a08a8d3586"
        },
        "date": 1776046212943,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 97.02104290587121,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 97.33251586932961,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 36.185546875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 36.5625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 123389.04716557995,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 123389.04716557995,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.005326,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2139787.3072780226,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 32430268.570551787,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.0531378984451294,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 50.25781817219228,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.39105620753553,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 30.203385416666666,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 42.6953125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 65822.49874504769,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 65139.19137739432,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00228,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 409919.78103285347,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 16608471.713809272,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.18039272611314,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.27526148601534,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.606640625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.7109375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 177849.6386417797,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 177849.6386417797,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002371,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 883998.2692321404,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 47578627.56957893,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 95.69162713723006,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 95.95993475728154,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 29.940625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 30.4375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 267943.81686210446,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 267943.81686210446,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002131,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2742168.563646446,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 71539484.57158226,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Network Utilization"
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
          "id": "c2e39e05477b8394ce45d84f73bd89f244a9419b",
          "message": "[query-engine] Slice expression bug fix (#2636)\n\n## Changes\n\n* Switch `Slice` expression (arrays and strings) to use `Slice(source,\nstart, [length])` instead of `Slice(source, start_inclusive,\n[end_exclusive])` to match KQL\n[substring](https://learn.microsoft.com/kusto/query/substring-function?view=azure-data-explorer)\nbehavior.\n\n/cc @albertlockett @drewrelmas\n\n---------\n\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-04-13T13:06:03Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/c2e39e05477b8394ce45d84f73bd89f244a9419b"
        },
        "date": 1776109010840,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 97.015420229293,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 97.29862827063013,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 35.61640625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 36.34375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 124443.69070488126,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 124443.69070488126,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003042,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2157263.700946937,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 32689924.34653207,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 17.626907348632812,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 0.2720568602827708,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 0.6209344931106151,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 34.13424479166667,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 35.734375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 5329.7821895678535,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 4408.153186806433,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002452,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 46958.85123542482,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 27704.80559903026,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 95.87973289165281,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.36720092915215,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 30.003645833333334,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 30.703125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 269419.5150905377,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 269419.5150905377,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002335,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2761683.328904451,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 72018807.70161897,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.709309458732605,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 33.56668424798045,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.31866954111489,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 36.667317708333336,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 42.98046875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 46298.22137666211,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 45521.584545793696,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002305,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 289139.5956506946,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11127256.544645661,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Network Utilization"
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
          "id": "ac4ee0b9fa2f3e68cea9c11a21c52372e50ad163",
          "message": "Fix typo in Prometheus test config filename (#2648)\n\n# Change Summary\nFix typo in test config filename\nfake-debug-noop-prometh'~~u~~'eus-telemetry.yaml ->\nfake-debug-noop-prometheus-telemetry.yaml\n\n## What issue does this PR close?\nminor nit\n\n## How are these changes tested?\nN/A\n\n## Are there any user-facing changes?\nN/A",
          "timestamp": "2026-04-13T23:38:17Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/ac4ee0b9fa2f3e68cea9c11a21c52372e50ad163"
        },
        "date": 1776133881506,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.19279620180154,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.38051274107352,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.907161458333334,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.7578125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 124312.031899744,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 124312.031899744,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002237,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 768358.5346305412,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 32697402.01739628,
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
            "value": 97.06414220546219,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 97.38825772720283,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 35.7125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 36.1796875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 124148.13355162056,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 124148.13355162056,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004124,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2156667.870248386,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 32605486.073494397,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.17981571921682,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.322949651433,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.702083333333334,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.2578125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 178506.54710943403,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 178506.54710943403,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002281,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 887760.6297886552,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 47808419.11788906,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 95.7796617787674,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.10100828622319,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 30.950130208333334,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 32.171875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 268218.35932320537,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 268218.35932320537,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002604,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2760273.310518792,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 71840499.37826866,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Cijo Thomas",
            "username": "cijothomas",
            "email": "cithomas@microsoft.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "ac4a60e9875c995eafa1f4a9e8f54c7d9ef10b3e",
          "message": "fix perf test (#2686)\n\nCloses #2667\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2026-04-16T02:51:58Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/ac4a60e9875c995eafa1f4a9e8f54c7d9ef10b3e"
        },
        "date": 1776361890953,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 94.7191219529458,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.96698733390318,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 35.536328125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 36.30078125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 123718.9041965631,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 123718.9041965631,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002148,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2141742.406015628,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 32514794.147294786,
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
            "value": 95.86094969165447,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.17806890159123,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 29.836458333333333,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 30.58984375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 269476.5363659833,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 269476.5363659833,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004111,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2763767.536083198,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 72005627.28093548,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 14.948860168457031,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 0.27735404276107906,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 0.4660165593376265,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 42.008984375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 43.29296875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 6333.06671122479,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5404.772459079472,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002526,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 46962.66521364714,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 27811.058216662983,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 22.560253143310547,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 0.2701879481448771,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 0.639603776505185,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 34.920703125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 35.71484375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 4196.498037387198,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3266.5354063855866,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002411,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 46963.65613570578,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 27442.132709769525,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Network Utilization"
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
          "id": "6ef19a99a26f17c89a12f5dad18c737259c1509d",
          "message": "run validation ci manually (#2675)\n\nI am seeing the validation tests acting up again in the validation ci\njob, setting the ci job to be a manual trigger instead of automatically\nrunning on every PR\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2026-04-16T20:11:07Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/6ef19a99a26f17c89a12f5dad18c737259c1509d"
        },
        "date": 1776392098967,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 93.63261214852602,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 94.5628104929196,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 30.579817708333334,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 30.92578125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 265284.94950053166,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 265284.94950053166,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003404,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2719489.117204066,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 71046105.00849333,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 97.01293206009474,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 97.3203492235185,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 35.8484375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 36.85546875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 126338.99148332601,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 126338.99148332601,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002062,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2192924.6635148395,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 33174242.56004036,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.20971383477774,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.5925912154311,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.117838541666668,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.91015625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 124965.56788785891,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 124965.56788785891,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002128,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 775316.3043502882,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 32908956.370916266,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 9.973752975463867,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 0.2807393578660027,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 0.48876508820798514,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 35.00703125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 35.90234375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 9502.976654942884,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 8574.678163746254,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002252,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 52583.17601880731,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 357058.7855972889,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Network Utilization"
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
          "id": "495588ee584201ea956c15c6fc102dc3465de675",
          "message": "update container config to allow configurable wait_for setting (#2672)\n\n# Change Summary\n\nUpdate ContainerConfig struct to have a wait_for field of type WaitFor\nfrom the test container crate. Added additional functions to allow a\nuser to configure the WaitFor enum variant to use for a test container\n\n## What issue does this PR close?\n\n* Closes #2668\n\n## How are these changes tested?\n\nunit tests\n\n## Are there any user-facing changes?\n\nno\n\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-04-17T12:32:22Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/495588ee584201ea956c15c6fc102dc3465de675"
        },
        "date": 1776447554856,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 7.875762939453125,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 0.2749762357299514,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 0.5003257328990228,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 35.31276041666667,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 35.91015625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 11997.904608208666,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 11072.937660360936,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002144,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 57051.25283568877,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 677711.4162598229,
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
            "value": 100.17358503666483,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.3304848954299,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.311328125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.05859375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 126397.43656578961,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 126397.43656578961,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002008,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 779652.0552699807,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 33371530.67211645,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 95.18472226265938,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 95.53540153333849,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 29.771875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 30.2578125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 275163.2085289712,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 275163.2085289712,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003298,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2812998.855015264,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 73954081.34626995,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.96074329499359,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 97.19277212457601,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 35.089453125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 35.60546875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 127180.65162197537,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 127180.65162197537,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003624,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2210956.059406386,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 33490409.68112344,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Network Utilization"
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
          "id": "0c13ab96cc8616b62a38d1916eb479b1114c3240",
          "message": "Update dependency pydantic to v2.13.2 (#2684)\n\nThis PR contains the following updates:\n\n| Package | Change |\n[Age](https://docs.renovatebot.com/merge-confidence/) |\n[Confidence](https://docs.renovatebot.com/merge-confidence/) |\n|---|---|---|---|\n| [pydantic](https://redirect.github.com/pydantic/pydantic)\n([changelog](https://docs.pydantic.dev/latest/changelog/)) | `==2.13.0`\n→ `==2.13.2` |\n![age](https://developer.mend.io/api/mc/badges/age/pypi/pydantic/2.13.2?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/pypi/pydantic/2.13.0/2.13.2?slim=true)\n|\n\n---\n\n### Release Notes\n\n<details>\n<summary>pydantic/pydantic (pydantic)</summary>\n\n###\n[`v2.13.2`](https://redirect.github.com/pydantic/pydantic/compare/v2.13.1...v2.13.2)\n\n###\n[`v2.13.1`](https://redirect.github.com/pydantic/pydantic/compare/v2.13.0...v2.13.1)\n\n[Compare\nSource](https://redirect.github.com/pydantic/pydantic/compare/v2.13.0...v2.13.1)\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: (UTC)\n\n- Branch creation\n  - \"before 8am every weekday\"\n- Automerge\n  - At any time (no schedule defined)\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My4xMjAuMiIsInVwZGF0ZWRJblZlciI6IjQzLjEyMy44IiwidGFyZ2V0QnJhbmNoIjoibWFpbiIsImxhYmVscyI6WyJkZXBlbmRlbmNpZXMiXX0=-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: Joshua MacDonald <jmacd@users.noreply.github.com>",
          "timestamp": "2026-04-18T00:27:24Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/0c13ab96cc8616b62a38d1916eb479b1114c3240"
        },
        "date": 1776481880522,
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
            "value": 100.13796164403996,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.32997530864198,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.139322916666668,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.23828125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 180618.78370352753,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 180618.78370352753,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002065,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 895482.6033775007,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 48325640.68032524,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 95.04658646842381,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.04039097975833,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 30.811328125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 31.2109375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 276712.17344588996,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 276712.17344588996,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.008202,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2809625.3987479145,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 73858255.63960451,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 97.00047077903453,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 97.30896466212027,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 35.91380208333333,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 36.484375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 125669.29971548887,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 125669.29971548887,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.007496,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2185845.574981593,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 33077441.286481302,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 9.506301879882812,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 1.1802836926284053,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 26.792031576503366,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 43.106640625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 44.3984375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 9499.64313007308,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 8614.676375324167,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002254,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 61174.739258981426,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 774265.7760936152,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Network Utilization"
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
          "id": "0c13ab96cc8616b62a38d1916eb479b1114c3240",
          "message": "Update dependency pydantic to v2.13.2 (#2684)\n\nThis PR contains the following updates:\n\n| Package | Change |\n[Age](https://docs.renovatebot.com/merge-confidence/) |\n[Confidence](https://docs.renovatebot.com/merge-confidence/) |\n|---|---|---|---|\n| [pydantic](https://redirect.github.com/pydantic/pydantic)\n([changelog](https://docs.pydantic.dev/latest/changelog/)) | `==2.13.0`\n→ `==2.13.2` |\n![age](https://developer.mend.io/api/mc/badges/age/pypi/pydantic/2.13.2?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/pypi/pydantic/2.13.0/2.13.2?slim=true)\n|\n\n---\n\n### Release Notes\n\n<details>\n<summary>pydantic/pydantic (pydantic)</summary>\n\n###\n[`v2.13.2`](https://redirect.github.com/pydantic/pydantic/compare/v2.13.1...v2.13.2)\n\n###\n[`v2.13.1`](https://redirect.github.com/pydantic/pydantic/compare/v2.13.0...v2.13.1)\n\n[Compare\nSource](https://redirect.github.com/pydantic/pydantic/compare/v2.13.0...v2.13.1)\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: (UTC)\n\n- Branch creation\n  - \"before 8am every weekday\"\n- Automerge\n  - At any time (no schedule defined)\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My4xMjAuMiIsInVwZGF0ZWRJblZlciI6IjQzLjEyMy44IiwidGFyZ2V0QnJhbmNoIjoibWFpbiIsImxhYmVscyI6WyJkZXBlbmRlbmNpZXMiXX0=-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: Joshua MacDonald <jmacd@users.noreply.github.com>",
          "timestamp": "2026-04-18T00:27:24Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/0c13ab96cc8616b62a38d1916eb479b1114c3240"
        },
        "date": 1776533261410,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 7.043343544006348,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 3.6187785040015727,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.28166924265842,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 41.22994791666667,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 43.34375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 12899.51476325299,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 12009.548240827007,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002257,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 81866.0768959121,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1708351.6964530454,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.15138288723597,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.30937640754833,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.170442708333333,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.2421875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 180836.75991711035,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 180836.75991711035,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002181,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 893727.7211850971,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 48454709.975627616,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 97.01492356926036,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 97.37144281707697,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 35.23463541666667,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 35.96484375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 125765.50597925301,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 125765.50597925301,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002144,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2183488.231183337,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 33071454.226620898,
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
            "value": 90.3139302318626,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 90.76441140024784,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 29.913411458333332,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 30.5546875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 265137.6032729913,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 265137.6032729913,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004314,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2695392.15353341,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 70939881.70826426,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Network Utilization"
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
          "id": "0c13ab96cc8616b62a38d1916eb479b1114c3240",
          "message": "Update dependency pydantic to v2.13.2 (#2684)\n\nThis PR contains the following updates:\n\n| Package | Change |\n[Age](https://docs.renovatebot.com/merge-confidence/) |\n[Confidence](https://docs.renovatebot.com/merge-confidence/) |\n|---|---|---|---|\n| [pydantic](https://redirect.github.com/pydantic/pydantic)\n([changelog](https://docs.pydantic.dev/latest/changelog/)) | `==2.13.0`\n→ `==2.13.2` |\n![age](https://developer.mend.io/api/mc/badges/age/pypi/pydantic/2.13.2?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/pypi/pydantic/2.13.0/2.13.2?slim=true)\n|\n\n---\n\n### Release Notes\n\n<details>\n<summary>pydantic/pydantic (pydantic)</summary>\n\n###\n[`v2.13.2`](https://redirect.github.com/pydantic/pydantic/compare/v2.13.1...v2.13.2)\n\n###\n[`v2.13.1`](https://redirect.github.com/pydantic/pydantic/compare/v2.13.0...v2.13.1)\n\n[Compare\nSource](https://redirect.github.com/pydantic/pydantic/compare/v2.13.0...v2.13.1)\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: (UTC)\n\n- Branch creation\n  - \"before 8am every weekday\"\n- Automerge\n  - At any time (no schedule defined)\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My4xMjAuMiIsInVwZGF0ZWRJblZlciI6IjQzLjEyMy44IiwidGFyZ2V0QnJhbmNoIjoibWFpbiIsImxhYmVscyI6WyJkZXBlbmRlbmNpZXMiXX0=-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: Joshua MacDonald <jmacd@users.noreply.github.com>",
          "timestamp": "2026-04-18T00:27:24Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/0c13ab96cc8616b62a38d1916eb479b1114c3240"
        },
        "date": 1776564717474,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.18969964577818,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.45971552257267,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.187239583333334,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.2421875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 125757.3699565265,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 125757.3699565265,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00205,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 779118.3037487812,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 33126746.386257395,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 95.06195341930436,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 95.36817907210904,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 30.116927083333334,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 31.01171875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 271579.18405784393,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 271579.18405784393,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003126,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2752545.198783161,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 72508902.59147136,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.83754764458799,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 97.12106452112023,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 35.468098958333336,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 35.7890625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 126532.68364900841,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 126532.68364900841,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002679,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2195531.9411625382,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 33223154.595631335,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.14607232380229,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.30367498262144,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.253385416666667,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.0546875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 178706.56333302573,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 178706.56333302573,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002273,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 869477.9919175537,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 47686259.97296863,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Network Utilization"
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
          "id": "0c13ab96cc8616b62a38d1916eb479b1114c3240",
          "message": "Update dependency pydantic to v2.13.2 (#2684)\n\nThis PR contains the following updates:\n\n| Package | Change |\n[Age](https://docs.renovatebot.com/merge-confidence/) |\n[Confidence](https://docs.renovatebot.com/merge-confidence/) |\n|---|---|---|---|\n| [pydantic](https://redirect.github.com/pydantic/pydantic)\n([changelog](https://docs.pydantic.dev/latest/changelog/)) | `==2.13.0`\n→ `==2.13.2` |\n![age](https://developer.mend.io/api/mc/badges/age/pypi/pydantic/2.13.2?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/pypi/pydantic/2.13.0/2.13.2?slim=true)\n|\n\n---\n\n### Release Notes\n\n<details>\n<summary>pydantic/pydantic (pydantic)</summary>\n\n###\n[`v2.13.2`](https://redirect.github.com/pydantic/pydantic/compare/v2.13.1...v2.13.2)\n\n###\n[`v2.13.1`](https://redirect.github.com/pydantic/pydantic/compare/v2.13.0...v2.13.1)\n\n[Compare\nSource](https://redirect.github.com/pydantic/pydantic/compare/v2.13.0...v2.13.1)\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: (UTC)\n\n- Branch creation\n  - \"before 8am every weekday\"\n- Automerge\n  - At any time (no schedule defined)\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My4xMjAuMiIsInVwZGF0ZWRJblZlciI6IjQzLjEyMy44IiwidGFyZ2V0QnJhbmNoIjoibWFpbiIsImxhYmVscyI6WyJkZXBlbmRlbmNpZXMiXX0=-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: Joshua MacDonald <jmacd@users.noreply.github.com>",
          "timestamp": "2026-04-18T00:27:24Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/0c13ab96cc8616b62a38d1916eb479b1114c3240"
        },
        "date": 1776619473991,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.83830182029205,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 97.1542219466832,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 35.273828125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 35.82421875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 125755.39316076388,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 125755.39316076388,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002198,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2187187.27324224,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 33047750.475356627,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.1718164179576,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.30547266780876,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.634765625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.19921875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 125499.02035033396,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 125499.02035033396,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002062,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 777383.147103193,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 33021940.65938308,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 89.93948954273418,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 90.46540254895866,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 28.8953125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 29.3203125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 257535.8777641867,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 257535.8777641867,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.008338,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2627885.4134483277,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 69028508.1476353,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.16758364742755,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.29076708507671,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.833203125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.93359375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 178559.82187349486,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 178559.82187349486,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.0023,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 880078.5021464762,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 47622980.43038437,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Network Utilization"
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
          "id": "0c13ab96cc8616b62a38d1916eb479b1114c3240",
          "message": "Update dependency pydantic to v2.13.2 (#2684)\n\nThis PR contains the following updates:\n\n| Package | Change |\n[Age](https://docs.renovatebot.com/merge-confidence/) |\n[Confidence](https://docs.renovatebot.com/merge-confidence/) |\n|---|---|---|---|\n| [pydantic](https://redirect.github.com/pydantic/pydantic)\n([changelog](https://docs.pydantic.dev/latest/changelog/)) | `==2.13.0`\n→ `==2.13.2` |\n![age](https://developer.mend.io/api/mc/badges/age/pypi/pydantic/2.13.2?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/pypi/pydantic/2.13.0/2.13.2?slim=true)\n|\n\n---\n\n### Release Notes\n\n<details>\n<summary>pydantic/pydantic (pydantic)</summary>\n\n###\n[`v2.13.2`](https://redirect.github.com/pydantic/pydantic/compare/v2.13.1...v2.13.2)\n\n###\n[`v2.13.1`](https://redirect.github.com/pydantic/pydantic/compare/v2.13.0...v2.13.1)\n\n[Compare\nSource](https://redirect.github.com/pydantic/pydantic/compare/v2.13.0...v2.13.1)\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: (UTC)\n\n- Branch creation\n  - \"before 8am every weekday\"\n- Automerge\n  - At any time (no schedule defined)\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My4xMjAuMiIsInVwZGF0ZWRJblZlciI6IjQzLjEyMy44IiwidGFyZ2V0QnJhbmNoIjoibWFpbiIsImxhYmVscyI6WyJkZXBlbmRlbmNpZXMiXX0=-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: Joshua MacDonald <jmacd@users.noreply.github.com>",
          "timestamp": "2026-04-18T00:27:24Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/0c13ab96cc8616b62a38d1916eb479b1114c3240"
        },
        "date": 1776650921986,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.18398329324829,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.3606258714175,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.042708333333334,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 38.3984375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 124475.53340294305,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 124475.53340294305,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002153,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 769634.6706360859,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 32753744.10560952,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.07064877022489,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.2495761989618,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.270182291666668,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 16.95703125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 180591.79892007576,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 180591.79892007576,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002171,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 874954.5654139094,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 48194464.967648946,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 95.74982501873073,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.11428217438439,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 30.309635416666666,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 30.7265625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 273309.1798348206,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 273309.1798348206,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.006766,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2805181.755406636,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 73087594.04900166,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.82896993737849,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 97.17055762081785,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 36.598307291666664,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 37.578125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 123953.5074101115,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 123953.5074101115,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002336,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2154459.8878116766,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 32589995.651905976,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Network Utilization"
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
          "id": "1e251126b4f960b4022cfa65b59ee21a50764def",
          "message": "test: Add JUnit XML upload and flaky test tracking workflow (#2699)\n\n# Change Summary\n\n- Implemented JUnit XML result uploads for both required and\nnon-required tests in the Rust-CI workflow.\n- Created a new workflow for detecting flaky tests from JUnit XML\nartifacts, which runs daily and on-demand.\n- The flaky test tracker parses JUnit results, identifies flaky tests,\nand creates or updates a tracking issue with a summary.\n\n## What issue does this PR close?\n\nn/a\n\n## How are these changes tested?\n\nn/a. This is adding a new flaky test detection workflow. No changes to\nproduct code.\n\n## Are there any user-facing changes?\n\nNo. Test/infra only.\n\n---------\n\nCo-authored-by: Lalit Kumar Bhasin <lalit_fin@yahoo.com>",
          "timestamp": "2026-04-20T18:48:02Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/1e251126b4f960b4022cfa65b59ee21a50764def"
        },
        "date": 1776714751408,
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
            "value": 100.14673604783266,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.33092527236458,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.385807291666666,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.2421875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 175402.03055370212,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 175402.03055370212,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002156,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 876537.1995567844,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 46786933.736665756,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.91448744933156,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 97.22226458963031,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 36.57838541666667,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 37.4375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 125118.72479363678,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 125118.72479363678,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00221,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2170951.291675032,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 32813897.489861146,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.21722977242418,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.44065492740191,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.876302083333332,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.66796875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 123999.42641036285,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 123999.42641036285,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001084,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 769218.8151232222,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 32605673.598508835,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 95.47178716806992,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.11126985356783,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 30.220572916666665,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 30.875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 268542.42944219906,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 268542.42944219906,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003181,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2727785.080180048,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 71727491.30184306,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Network Utilization"
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
          "id": "5a0d3fb508582e0922747323347ab66c85351dd3",
          "message": "Update docker.io/rust Docker tag to v1.95 (#2707)\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n|---|---|---|---|\n| docker.io/rust | stage | minor | `1.94` → `1.95` |\n\n---\n\n### Configuration\n\n📅 **Schedule**: (UTC)\n\n- Branch creation\n  - \"before 8am on Monday\"\n- Automerge\n  - At any time (no schedule defined)\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My4xMjMuOCIsInVwZGF0ZWRJblZlciI6IjQzLjEyMy44IiwidGFyZ2V0QnJhbmNoIjoibWFpbiIsImxhYmVscyI6WyJkZXBlbmRlbmNpZXMiXX0=-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: Drew Relmas <drewrelmas@gmail.com>",
          "timestamp": "2026-04-20T21:52:46Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/5a0d3fb508582e0922747323347ab66c85351dd3"
        },
        "date": 1776737825658,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 90.2248508079349,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 90.62074945803654,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 29.556770833333335,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 30.02734375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 260475.9986556228,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 260475.9986556228,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003993,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2652488.104037267,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 69629497.43908814,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 93.26332079385591,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.8909347169519,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 35.355859375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 35.9609375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 123131.84980074065,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 123131.84980074065,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00722,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2123459.562408443,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 32276496.606473565,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 19.015735626220703,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 0.2841064614074958,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 0.5985189784987969,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 35.33515625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 36.578125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 4956.467334072048,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 4031.5045329926984,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002413,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 46938.14149692766,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 28204.33229998711,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 3.1591005325317383,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 16.915043551222794,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.53130170600608,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 42.50950520833333,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 45.54296875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 27625.653265619374,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 26770.68462871887,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002201,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 173877.6277200669,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5885793.014948297,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Network Utilization"
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
          "id": "693b34e1972d186270e39913fdadd6b8c3751bf9",
          "message": "Transport header support in OTLP/Fake Data receivers (#2702)\n\n# Change Summary\n\nUpdate the OTLP receiver to use the capture header policy when defined,\nextracts data from grpc/http headers\n\nUpdate the Fake Data Generator to allow user to specify transport\nheaders to tack on to generated OtapPdata, users can provide key/value\npairs or just the key (random value will be generated).\n\n## What issue does this PR close?\n\n* Closes #2692\n\n## How are these changes tested?\n\nunit tests\n\n## Are there any user-facing changes?\n\nno\n\n---------\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2026-04-21T17:41:03Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/693b34e1972d186270e39913fdadd6b8c3751bf9"
        },
        "date": 1776795336022,
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
            "value": 100.13717649980694,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.34450673687472,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.171744791666665,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.1015625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 180595.12009356328,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 180595.12009356328,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002175,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 880461.9538308196,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 48267440.90410527,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 93.15294943433764,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.97262929363418,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 36.00533854166667,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 36.72265625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 122891.96059538209,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 122891.96059538209,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00718,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2123629.580970064,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 32287156.07779286,
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
            "value": 95.71532557228039,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.07791047076046,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 31.184244791666668,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 31.671875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 271459.43055734975,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 271459.43055734975,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001231,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2779913.492013529,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 72553779.34787557,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.16441709480488,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.3780504515727,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.7703125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.33203125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 125225.24352783333,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 125225.24352783333,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002279,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 772932.4193173822,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 32949330.17266329,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Network Utilization"
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
          "id": "dd344850b0de4426753c6c5ac7ca8786a2545458",
          "message": "[query-engine] Tweak slice validation errors (#2721)\n\nRelates to #2636\n\n# Changes\n\n* Tweak the slice validation error messages\n\n---------\n\nCo-authored-by: Drew Relmas <drewrelmas@gmail.com>",
          "timestamp": "2026-04-21T19:39:49Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/dd344850b0de4426753c6c5ac7ca8786a2545458"
        },
        "date": 1776824207814,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.1863520996136,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.40174220854175,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.057291666666668,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.7734375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 125725.59541330735,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 125725.59541330735,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002102,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 775419.9989064016,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 33136721.82473952,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.13562394380261,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.30692385157643,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.091015625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.98046875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 179078.53526487693,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 179078.53526487693,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002166,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 867235.2951887015,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 47773362.323898755,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 95.62212811699915,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 95.87584455798111,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 30.607942708333333,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 31.375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 270667.2151006741,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 270667.2151006741,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003573,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2758656.1603352255,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 72435921.02413116,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 93.25275619975918,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.94392671376445,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 35.53294270833333,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 36.78515625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 122847.22778683597,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 122847.22778683597,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002168,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2121587.708229827,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 32269427.00525565,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Network Utilization"
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
          "id": "3a6d2a3b70b838918f335bbc21b28c0fdab54f25",
          "message": "fix(benchmarks): Bump max_decoding_message_size to 32MiB to fix batch processor benchmarks (#2730)\n\n# Change Summary\n\nBatch processor benchmarks had 100% signal drop rate due to being over\nthe decompression limit on the backend engine. Bumping the limit fixes\nthe issue for both continuous and nightly.\n\n## What issue does this PR close?\n\n* Closes #2729\n\n## How are these changes tested?\n\nRan all scenarios locally and observed the dropped rate being 0 (or\nless):\n\n<img width=\"1891\" height=\"466\" alt=\"image\"\nsrc=\"https://github.com/user-attachments/assets/6af43bf9-2e61-4af9-a48b-8285f8768a92\"\n/>\n\n## Are there any user-facing changes?\n\nNo",
          "timestamp": "2026-04-22T15:57:57Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/3a6d2a3b70b838918f335bbc21b28c0fdab54f25"
        },
        "date": 1776881466276,
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
            "value": 100.13279211704796,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.29863841719894,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.584114583333335,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.80859375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 179198.70116565403,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 179198.70116565403,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002109,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 868628.8253833848,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 47913369.2810681,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.92024756589646,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 97.39248605083695,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 35.50833333333333,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 35.8046875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 125933.80391418589,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 125933.80391418589,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002158,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2187657.4987520785,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 33066416.562286947,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.17838668372585,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.3796729949632,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.575130208333334,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.4453125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 125414.86635147067,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 125414.86635147067,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002456,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 773754.3171621788,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 32972703.687289193,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 95.68628760250282,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.06660566110897,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 30.637109375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 31.1640625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 266778.87345565954,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 266778.87345565954,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.005876,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2736450.389279923,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 71326666.5191316,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Network Utilization"
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
          "id": "446e2510d411d9944a3a37faba577e2029677e8b",
          "message": "feat(config): add optional field to EngineConfig (#2727)\n\nAllow applications embedding the dataflow engine to carry their own\nengine-level configuration under `engine.custom`. The engine ignores\nthis field entirely; embedding binaries can read namespaced keys for\nconcerns like remote management, auth, or fleet coordination.\n\n# Change Summary\n\nAdd an optional `custom: HashMap<String, serde_json::Value>` field to\n`EngineConfig`. This gives embedding applications an escape hatch for\nengine-level config without forking the config crate or pre-parsing\nYAML. The field defaults to an empty map and is omitted from serialized\noutput when empty.\n\n## What issue does this PR close?\n\n* Closes #2561\n\n## How are these changes tested?\n\nThree new unit tests in `engine.rs`:\n- `from_yaml_accepts_custom_config` — parses a config with multiple\nnamespaced custom keys and verifies values\n- `custom_defaults_to_empty` — confirms the field defaults to an empty\nmap when omitted\n- `custom_roundtrips_through_json` — serializes to JSON and deserializes\nback, verifying data is preserved\n\n## Are there any user-facing changes?\n\nYes. A new optional `custom` key is available under the `engine` section\nof the YAML/JSON config. Example:\n\n```yaml\nengine:\n  custom:\n    remote_management:\n      server_url: \"ws://mgmt.example.com/v1\"\n      heartbeat_interval_secs: 10\n    custom_auth:\n      provider: \"oidc\"\n      token_endpoint: \"https://auth.example.com/token\"\n```\n\nExisting configs are unaffected since the field defaults to empty.\n\n---------\n\nCo-authored-by: Lalit Kumar Bhasin <lalit_fin@yahoo.com>",
          "timestamp": "2026-04-22T23:05:42Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/446e2510d411d9944a3a37faba577e2029677e8b"
        },
        "date": 1776911181046,
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
            "value": 100.04042434193197,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.15685034488104,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.588020833333335,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.1953125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 179893.46087269727,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 179893.46087269727,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002181,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 873226.5108908025,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 48021063.56956208,
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
            "value": 100.21877911648222,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.3559466997211,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.6984375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.1953125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 125179.06472722614,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 125179.06472722614,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002046,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 778282.2566047,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 33056929.633235972,
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
            "value": 97.04341520646405,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 97.4297893432466,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 35.492578125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 36.32421875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 126170.27702596333,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 126170.27702596333,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002246,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2191991.6401287066,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 33171615.432391644,
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
            "value": 96.13451745682234,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.37689054148268,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 30.657421875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 31.28125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 273273.0856598346,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 273273.0856598346,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.007739,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2817886.360309982,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 73224741.9852381,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Sameer J",
            "username": "sjmsft",
            "email": "101909410+sjmsft@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "9c54c8e469806b54ec6fa8b9721790c951ca0a97",
          "message": "Task 1614: Fix tests that are ignored on Windows and MacOS (#2724)\n\n# Change Summary\n\nFix tests that are ignored on Windows and MacOS\n\n## What issue does this PR close?\n\n* Closes #1614\n\n## How are these changes tested?\n\nBy running the reenabled tests.\n\n## Are there any user-facing changes?\n\nYes, one production code change:\n\nWindows CA certificate hot-reload fix - The get_file_identity() function\nnow uses MetadataExt::last_write_time() (100ns-precision FILETIME) on\nWindows instead of the platform-fallback get_mtime() which truncated to\nwhole seconds. Previously, if a CA certificate file was replaced within\nthe same second (e.g., by automation or cert-manager), the file watcher\ncould miss the change entirely, leaving stale CA certificates in memory\nuntil the next reload event. This affects any Windows deployment using\nmTLS with watch_client_ca: true.\n\nAll other changes are test-only (removing #[ignore] attributes and\nfixing test assertions for cross-platform compatibility).",
          "timestamp": "2026-04-23T17:12:49Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/9c54c8e469806b54ec6fa8b9721790c951ca0a97"
        },
        "date": 1776969961312,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.95423286598715,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 97.44251607405687,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 36.197265625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 37.1015625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 0,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 115530.68382965084,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001376,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2175211.0036441577,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 32902392.39215925,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.21536453322311,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.3973253030654,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.495703125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.26171875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 0,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 164300.70802765552,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002176,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 859029.7472014613,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 47324011.3826265,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.33412338201566,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 97.48106062955497,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 30.018359375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 30.55859375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 0,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 243879.6498004878,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.009107,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2728038.5823380887,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 71003925.98015471,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.20688291842077,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.35822702702704,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.930078125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.19140625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 0,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 115100.97530256359,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002098,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 772385.776522203,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 32769343.077400815,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "sapatrjv",
            "username": "sapatrjv",
            "email": "sapatrjv@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "da7688be82431d7cf4508c7376036fb240785034",
          "message": "Use latest weaver build and simplify crypto selection. (#2740)\n\n# Change Summary\n\n<!--\nUse latest weaver build that has the exclusion of openssl build in case\nof windows platforms. In case of windows platforms it uses SChannel TLS\ninstead of natively building openssl.\n\nSimplification of crypto selection.\n\n-->\n\n## What issue does this PR close?\n\nPart of https://github.com/open-telemetry/otel-arrow/issues/2697\n\n## How are these changes tested?\n\nSearch cargo tree and check on windows platform no openssl dependency.\n\n## Are there any user-facing changes?\n\n <!-- If yes, provide further info below -->\n\n---------\n\nCo-authored-by: Lalit Kumar Bhasin <lalit_fin@yahoo.com>\nCo-authored-by: Joshua MacDonald <jmacd@users.noreply.github.com>",
          "timestamp": "2026-04-24T00:33:27Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/da7688be82431d7cf4508c7376036fb240785034"
        },
        "date": 1776999387007,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.9857372318397,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 97.283987849521,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 35.32057291666667,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 35.56640625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 0,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 115129.26927012809,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002118,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2206569.503679056,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 33344592.98757601,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.104659927191,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.32054539820265,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.682552083333334,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.2265625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 0,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 165451.01375702998,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00205,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 874394.3621333324,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 47770584.89673224,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 95.6140100473058,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 95.92408371302803,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 30.028255208333334,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 30.83984375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 0,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 248135.7735489832,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00344,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2799211.4114477932,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 72925577.91831248,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.2021766198559,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.28813874264478,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.488020833333334,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 38.24609375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 0,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 117080.5314263839,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00229,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 782393.1673793981,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 33271070.37215297,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Marc Snider",
            "username": "marcsnid",
            "email": "30638925+marcsnid@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "bc4cec8b9dd31adbf3a207903eb652c16499467c",
          "message": "Added debug assertions for negative values in counter and histograms (#2757)\n\n# Change Summary\n\nAdd `debug_assert!` checks to enforce non-negative values in\n`Counter<f64>` and `Mmsc` (Histogram bridge) instruments in\n`otap_df_telemetry`. Counters and Histogram-based instruments must only\nreceive non-negative deltas for correctness. Their sums are exported as\nPrometheus counters, which require monotonicity.\n\nThree guards added:\n- `Counter<f64>::add(v)` — asserts `v >= 0.0`\n- `AddAssign<f64> for Counter<f64>` — asserts `rhs >= 0.0`\n- `Mmsc::record(value)` — asserts `value >= 0.0`\n\nThese use `debug_assert!` (zero cost in release builds) per the issue\ndiscussion.\n\n## What issue does this PR close?\n#2100\n\n## How are these changes tested?\n\n- Replaced the existing `test_mmsc_negative_values` test (which\nvalidated now-invalid behavior) with three `#[cfg(debug_assertions)]\n#[should_panic]` tests that verify the assertions fire on negative\ninput:\n  - `test_mmsc_record_rejects_negative`\n  - `test_counter_f64_add_rejects_negative`\n  - `test_counter_f64_add_assign_rejects_negative`\n- Tests in `otap-df-telemetry` continue to pass.\n\n## Are there any user-facing changes?\n\nNo. `debug_assert!` is stripped in release builds. In debug builds,\npassing a negative value to `Counter<f64>::add()`, `Counter<f64> +=`, or\n`Mmsc::record()` will now panic with a descriptive message, catching\nincorrect usage during development.",
          "timestamp": "2026-04-24T23:10:51Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/bc4cec8b9dd31adbf3a207903eb652c16499467c"
        },
        "date": 1777080042234,
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
            "value": 0.28467125281742794,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 0.6584999614584136,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 34.14296875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 35.1484375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 0,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 0,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002331,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 49437.65673235084,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 28136.324428897122,
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
            "value": 100.20816541977466,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.41923272084263,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.169140625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.1484375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 0,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 113302.62365137937,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002141,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 774483.9680012724,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 32766490.980800923,
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
            "value": 96.91314558280925,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 97.22430067775724,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 35.688411458333334,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 36.43359375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 0,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 114895.51141535403,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002344,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2176511.5384412697,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 32960413.278865423,
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
            "value": 95.80358369623184,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.224850127798,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 29.356770833333332,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 29.84375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 0,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 245217.30429554253,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003922,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2719183.260859129,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 71456391.96866384,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Marc Snider",
            "username": "marcsnid",
            "email": "30638925+marcsnid@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "bc4cec8b9dd31adbf3a207903eb652c16499467c",
          "message": "Added debug assertions for negative values in counter and histograms (#2757)\n\n# Change Summary\n\nAdd `debug_assert!` checks to enforce non-negative values in\n`Counter<f64>` and `Mmsc` (Histogram bridge) instruments in\n`otap_df_telemetry`. Counters and Histogram-based instruments must only\nreceive non-negative deltas for correctness. Their sums are exported as\nPrometheus counters, which require monotonicity.\n\nThree guards added:\n- `Counter<f64>::add(v)` — asserts `v >= 0.0`\n- `AddAssign<f64> for Counter<f64>` — asserts `rhs >= 0.0`\n- `Mmsc::record(value)` — asserts `value >= 0.0`\n\nThese use `debug_assert!` (zero cost in release builds) per the issue\ndiscussion.\n\n## What issue does this PR close?\n#2100\n\n## How are these changes tested?\n\n- Replaced the existing `test_mmsc_negative_values` test (which\nvalidated now-invalid behavior) with three `#[cfg(debug_assertions)]\n#[should_panic]` tests that verify the assertions fire on negative\ninput:\n  - `test_mmsc_record_rejects_negative`\n  - `test_counter_f64_add_rejects_negative`\n  - `test_counter_f64_add_assign_rejects_negative`\n- Tests in `otap-df-telemetry` continue to pass.\n\n## Are there any user-facing changes?\n\nNo. `debug_assert!` is stripped in release builds. In debug builds,\npassing a negative value to `Counter<f64>::add()`, `Counter<f64> +=`, or\n`Mmsc::record()` will now panic with a descriptive message, catching\nincorrect usage during development.",
          "timestamp": "2026-04-24T23:10:51Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/bc4cec8b9dd31adbf3a207903eb652c16499467c"
        },
        "date": 1777135285066,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 95.83445931264829,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.12058114374034,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 29.616796875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 30.09765625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 0,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 245637.00989612666,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003173,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2745994.657610881,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 71494816.67676803,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.18343049777663,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.44436459382018,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.705598958333333,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 39.44140625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 0,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 114283.89163231371,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.006707,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 778633.8572364076,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 32975071.292908616,
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
            "value": 93.06574690963019,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 93.4612756088133,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 34.81809895833333,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 35.86328125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 0,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 112203.86948939215,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002387,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2123584.5582485846,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 32218359.959925216,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.16607068844598,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.41016383307573,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.434375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.0546875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 0,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 161605.81653610806,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002172,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 864043.6580185445,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 47412915.493103296,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Marc Snider",
            "username": "marcsnid",
            "email": "30638925+marcsnid@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "bc4cec8b9dd31adbf3a207903eb652c16499467c",
          "message": "Added debug assertions for negative values in counter and histograms (#2757)\n\n# Change Summary\n\nAdd `debug_assert!` checks to enforce non-negative values in\n`Counter<f64>` and `Mmsc` (Histogram bridge) instruments in\n`otap_df_telemetry`. Counters and Histogram-based instruments must only\nreceive non-negative deltas for correctness. Their sums are exported as\nPrometheus counters, which require monotonicity.\n\nThree guards added:\n- `Counter<f64>::add(v)` — asserts `v >= 0.0`\n- `AddAssign<f64> for Counter<f64>` — asserts `rhs >= 0.0`\n- `Mmsc::record(value)` — asserts `value >= 0.0`\n\nThese use `debug_assert!` (zero cost in release builds) per the issue\ndiscussion.\n\n## What issue does this PR close?\n#2100\n\n## How are these changes tested?\n\n- Replaced the existing `test_mmsc_negative_values` test (which\nvalidated now-invalid behavior) with three `#[cfg(debug_assertions)]\n#[should_panic]` tests that verify the assertions fire on negative\ninput:\n  - `test_mmsc_record_rejects_negative`\n  - `test_counter_f64_add_rejects_negative`\n  - `test_counter_f64_add_assign_rejects_negative`\n- Tests in `otap-df-telemetry` continue to pass.\n\n## Are there any user-facing changes?\n\nNo. `debug_assert!` is stripped in release builds. In debug builds,\npassing a negative value to `Counter<f64>::add()`, `Counter<f64> +=`, or\n`Mmsc::record()` will now panic with a descriptive message, catching\nincorrect usage during development.",
          "timestamp": "2026-04-24T23:10:51Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/bc4cec8b9dd31adbf3a207903eb652c16499467c"
        },
        "date": 1777166640508,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 90.15883966555285,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 90.44860947661815,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 29.868489583333332,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 30.5078125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 0,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 234262.81052509235,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003122,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2608718.7807576195,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 68265782.49593644,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.19751958933412,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.42859219172786,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.513802083333335,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.3515625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 0,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 114251.02406436256,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002088,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 779690.2207893557,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 33049392.40053932,
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
            "value": 93.37757773712308,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 97.25173345687593,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 35.907421875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 36.73046875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 0,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 112278.228416548,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002728,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2124934.947814125,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 32236100.015668545,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.1772743088305,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.3383850931677,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.10234375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 16.71484375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 0,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 163634.12008061845,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002156,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 879510.2901617045,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 47221919.05283513,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Marc Snider",
            "username": "marcsnid",
            "email": "30638925+marcsnid@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "bc4cec8b9dd31adbf3a207903eb652c16499467c",
          "message": "Added debug assertions for negative values in counter and histograms (#2757)\n\n# Change Summary\n\nAdd `debug_assert!` checks to enforce non-negative values in\n`Counter<f64>` and `Mmsc` (Histogram bridge) instruments in\n`otap_df_telemetry`. Counters and Histogram-based instruments must only\nreceive non-negative deltas for correctness. Their sums are exported as\nPrometheus counters, which require monotonicity.\n\nThree guards added:\n- `Counter<f64>::add(v)` — asserts `v >= 0.0`\n- `AddAssign<f64> for Counter<f64>` — asserts `rhs >= 0.0`\n- `Mmsc::record(value)` — asserts `value >= 0.0`\n\nThese use `debug_assert!` (zero cost in release builds) per the issue\ndiscussion.\n\n## What issue does this PR close?\n#2100\n\n## How are these changes tested?\n\n- Replaced the existing `test_mmsc_negative_values` test (which\nvalidated now-invalid behavior) with three `#[cfg(debug_assertions)]\n#[should_panic]` tests that verify the assertions fire on negative\ninput:\n  - `test_mmsc_record_rejects_negative`\n  - `test_counter_f64_add_rejects_negative`\n  - `test_counter_f64_add_assign_rejects_negative`\n- Tests in `otap-df-telemetry` continue to pass.\n\n## Are there any user-facing changes?\n\nNo. `debug_assert!` is stripped in release builds. In debug builds,\npassing a negative value to `Counter<f64>::add()`, `Counter<f64> +=`, or\n`Mmsc::record()` will now panic with a descriptive message, catching\nincorrect usage during development.",
          "timestamp": "2026-04-24T23:10:51Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/bc4cec8b9dd31adbf3a207903eb652c16499467c"
        },
        "date": 1777221735476,
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
            "value": 0.2760472785902322,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 0.6111854197235308,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 35.95729166666667,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 36.7578125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 0,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 0,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002196,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 49484.26423856002,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 27535.122946009444,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 95.75737754121187,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.04989207236588,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 29.825130208333334,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 30.796875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 0,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 245352.62022915852,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003435,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2765389.754769011,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 71433344.52438742,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 93.18619803565078,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 95.76545538889748,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 34.86575520833333,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 35.53125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 0,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 111154.5170282487,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00206,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2120993.955245101,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 32165155.74950642,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.2151752771092,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.41887249572916,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.535546875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.23828125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 0,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 113961.02465958979,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002093,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 777694.5114455994,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 32927435.920500673,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Marc Snider",
            "username": "marcsnid",
            "email": "30638925+marcsnid@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "bc4cec8b9dd31adbf3a207903eb652c16499467c",
          "message": "Added debug assertions for negative values in counter and histograms (#2757)\n\n# Change Summary\n\nAdd `debug_assert!` checks to enforce non-negative values in\n`Counter<f64>` and `Mmsc` (Histogram bridge) instruments in\n`otap_df_telemetry`. Counters and Histogram-based instruments must only\nreceive non-negative deltas for correctness. Their sums are exported as\nPrometheus counters, which require monotonicity.\n\nThree guards added:\n- `Counter<f64>::add(v)` — asserts `v >= 0.0`\n- `AddAssign<f64> for Counter<f64>` — asserts `rhs >= 0.0`\n- `Mmsc::record(value)` — asserts `value >= 0.0`\n\nThese use `debug_assert!` (zero cost in release builds) per the issue\ndiscussion.\n\n## What issue does this PR close?\n#2100\n\n## How are these changes tested?\n\n- Replaced the existing `test_mmsc_negative_values` test (which\nvalidated now-invalid behavior) with three `#[cfg(debug_assertions)]\n#[should_panic]` tests that verify the assertions fire on negative\ninput:\n  - `test_mmsc_record_rejects_negative`\n  - `test_counter_f64_add_rejects_negative`\n  - `test_counter_f64_add_assign_rejects_negative`\n- Tests in `otap-df-telemetry` continue to pass.\n\n## Are there any user-facing changes?\n\nNo. `debug_assert!` is stripped in release builds. In debug builds,\npassing a negative value to `Counter<f64>::add()`, `Counter<f64> +=`, or\n`Mmsc::record()` will now panic with a descriptive message, catching\nincorrect usage during development.",
          "timestamp": "2026-04-24T23:10:51Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/bc4cec8b9dd31adbf3a207903eb652c16499467c"
        },
        "date": 1777253043560,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 93.13113567455514,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 93.5861101215829,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 35.827734375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 36.3828125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 0,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 112254.2416662246,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002187,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2122882.1004196233,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 32210104.15261654,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.21416822717386,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.40807145631068,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.58125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.45703125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 0,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 114420.45369397322,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002384,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 774967.043203072,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 33092970.11226236,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.16176070861364,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.36952528459693,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 15.977083333333333,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 16.66015625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 0,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 164210.8809699605,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002114,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 873203.1022420281,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 47365693.616585,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 90.02104468989901,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 90.2329995378216,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 29.550651041666665,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 30.546875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 0,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 231739.2859203476,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002774,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2572716.5150471744,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 67438124.15751909,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Marc Snider",
            "username": "marcsnid",
            "email": "30638925+marcsnid@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "bc4cec8b9dd31adbf3a207903eb652c16499467c",
          "message": "Added debug assertions for negative values in counter and histograms (#2757)\n\n# Change Summary\n\nAdd `debug_assert!` checks to enforce non-negative values in\n`Counter<f64>` and `Mmsc` (Histogram bridge) instruments in\n`otap_df_telemetry`. Counters and Histogram-based instruments must only\nreceive non-negative deltas for correctness. Their sums are exported as\nPrometheus counters, which require monotonicity.\n\nThree guards added:\n- `Counter<f64>::add(v)` — asserts `v >= 0.0`\n- `AddAssign<f64> for Counter<f64>` — asserts `rhs >= 0.0`\n- `Mmsc::record(value)` — asserts `value >= 0.0`\n\nThese use `debug_assert!` (zero cost in release builds) per the issue\ndiscussion.\n\n## What issue does this PR close?\n#2100\n\n## How are these changes tested?\n\n- Replaced the existing `test_mmsc_negative_values` test (which\nvalidated now-invalid behavior) with three `#[cfg(debug_assertions)]\n#[should_panic]` tests that verify the assertions fire on negative\ninput:\n  - `test_mmsc_record_rejects_negative`\n  - `test_counter_f64_add_rejects_negative`\n  - `test_counter_f64_add_assign_rejects_negative`\n- Tests in `otap-df-telemetry` continue to pass.\n\n## Are there any user-facing changes?\n\nNo. `debug_assert!` is stripped in release builds. In debug builds,\npassing a negative value to `Counter<f64>::add()`, `Counter<f64> +=`, or\n`Mmsc::record()` will now panic with a descriptive message, catching\nincorrect usage during development.",
          "timestamp": "2026-04-24T23:10:51Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/bc4cec8b9dd31adbf3a207903eb652c16499467c"
        },
        "date": 1777309265011,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 95.7788437336998,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.15961606935521,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 31.24609375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 31.79296875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 0,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 245794.19607048936,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003451,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2760940.5962161184,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 71534537.02636476,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.98321300415283,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 97.21044662899605,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 35.44713541666667,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 35.9921875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 0,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 114029.02683708645,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002266,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2183601.9477331047,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 33000012.689673685,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.2312522634624,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.47753808278198,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.018489583333334,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 39.484375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 0,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 114604.36229217991,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002079,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 769549.4102696008,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 32581995.471066494,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.13560874436966,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.24033979457873,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.211328125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.13671875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 0,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 160857.52101490734,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002168,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 886903.6342347491,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 47198849.97461219,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Network Utilization"
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
          "id": "d5f4b35d5509da5dbe73e48f914d9220fc2e1a3d",
          "message": "Fix batch byte sizing and wakeup state (#2763)\n\n# Change Summary\n\nFixes batch processor behavior for OTLP byte-sized batches by comparing\n`min_size` against pending bytes instead of item count. Also avoids\nunnecessary wakeup set/cancel work for immediate size flushes, clears\nstale timer state after full drains, and adds wakeup scheduler metrics\nfor attribution.\n\n## What issue does this PR close?\n\n* Closes #NNN\n\n## How are these changes tested?\n\n- cargo xtask check\n- controlled benchmark\n\n## Are there any user-facing changes?",
          "timestamp": "2026-04-27T22:13:59Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/d5f4b35d5509da5dbe73e48f914d9220fc2e1a3d"
        },
        "date": 1777340114980,
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
            "value": 100.18719068180242,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.48719993793156,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.254296875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.0078125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 0,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 166137.40776578969,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00214,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 889693.0297938496,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 47960546.593485065,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 97.02485242383042,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 97.32689334675858,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 35.16953125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 35.8203125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 0,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 114845.84258049859,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002172,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2197535.788042921,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 33217640.554605324,
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
            "value": 95.70422452271234,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 95.98542863780686,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 31.0734375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 31.65234375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 0,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 247187.47171497523,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003041,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2788679.8837361867,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 72585924.64646007,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 43.59965583148661,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.4094175960347,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 30.348567708333334,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.05859375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 0,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 50558.14620130595,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.0022,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 373695.2505860955,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 14867629.223793382,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Network Utilization"
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
          "id": "9eab7837283aaa7f64ea77ae318fbb236d1f3794",
          "message": "Update Rust crate zip to v8 (#2762)\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n|---|---|---|---|\n| [zip](https://redirect.github.com/zip-rs/zip2) |\nworkspace.dependencies | major | `=4.6.1` → `=8.6.0` |\n\n---\n\n### Release Notes\n\n<details>\n<summary>zip-rs/zip2 (zip)</summary>\n\n###\n[`v8.6.0`](https://redirect.github.com/zip-rs/zip2/blob/HEAD/CHANGELOG.md#860---2026-04-25)\n\n[Compare\nSource](https://redirect.github.com/zip-rs/zip2/compare/v8.5.1...v8.6.0)\n\n##### <!-- 0 -->🚀 Features\n\n- add `compression not supported` as enum error\n([#&#8203;774](https://redirect.github.com/zip-rs/zip2/pull/774))\n\n##### <!-- 1 -->🐛 Bug Fixes\n\n- allow for `[u8]` as filename\n([#&#8203;775](https://redirect.github.com/zip-rs/zip2/pull/775))\n\n##### <!-- 2 -->🚜 Refactor\n\n- mark `ZipFlags` as non-exhaustive and add test for `HasZipMetadata`\n([#&#8203;777](https://redirect.github.com/zip-rs/zip2/pull/777))\n- use and simplify is\\_dir\n([#&#8203;776](https://redirect.github.com/zip-rs/zip2/pull/776))\n\n###\n[`v8.5.1`](https://redirect.github.com/zip-rs/zip2/blob/HEAD/CHANGELOG.md#851---2026-04-06)\n\n[Compare\nSource](https://redirect.github.com/zip-rs/zip2/compare/v8.5.0...v8.5.1)\n\n##### <!-- 2 -->🚜 Refactor\n\n- change magic finder to stack buffer\n([#&#8203;763](https://redirect.github.com/zip-rs/zip2/pull/763))\n- simplify extra field parsing\n([#&#8203;764](https://redirect.github.com/zip-rs/zip2/pull/764))\n\n###\n[`v8.5.0`](https://redirect.github.com/zip-rs/zip2/blob/HEAD/CHANGELOG.md#850---2026-04-01)\n\n[Compare\nSource](https://redirect.github.com/zip-rs/zip2/compare/v8.4.0...v8.5.0)\n\n##### <!-- 1 -->🐛 Bug Fixes\n\n- remove `zip64 comment` and add `zip64 extensible data sector`\n([#&#8203;747](https://redirect.github.com/zip-rs/zip2/pull/747))\n\n##### <!-- 2 -->🚜 Refactor\n\n- remove useless magic in struct\n([#&#8203;730](https://redirect.github.com/zip-rs/zip2/pull/730))\n- change extra\\_field from Arc\\<Vec<u8>> to Arc<\\[u8]>\n([#&#8203;741](https://redirect.github.com/zip-rs/zip2/pull/741))\n\n##### <!-- 7 -->⚙️ Miscellaneous Tasks\n\n- cleanup README\n([#&#8203;758](https://redirect.github.com/zip-rs/zip2/pull/758))\n\n###\n[`v8.4.0`](https://redirect.github.com/zip-rs/zip2/blob/HEAD/CHANGELOG.md#840---2026-03-23)\n\n[Compare\nSource](https://redirect.github.com/zip-rs/zip2/compare/v8.3.1...v8.4.0)\n\n##### <!-- 0 -->🚀 Features\n\n- add a check for building benches\n([#&#8203;748](https://redirect.github.com/zip-rs/zip2/pull/748))\n\n##### <!-- 2 -->🚜 Refactor\n\n- split part of `read.rs` for code readability\n([#&#8203;744](https://redirect.github.com/zip-rs/zip2/pull/744))\n- remove unused allow\n([#&#8203;745](https://redirect.github.com/zip-rs/zip2/pull/745))\n\n##### <!-- 4 -->⚡ Performance\n\n- skip BufReader for Stored files in make\\_reader\n([#&#8203;739](https://redirect.github.com/zip-rs/zip2/pull/739))\n\n##### <!-- 7 -->⚙️ Miscellaneous Tasks\n\n- move pull request template to correct folder\n([#&#8203;749](https://redirect.github.com/zip-rs/zip2/pull/749))\n\n###\n[`v8.3.1`](https://redirect.github.com/zip-rs/zip2/blob/HEAD/CHANGELOG.md#831---2026-03-21)\n\n[Compare\nSource](https://redirect.github.com/zip-rs/zip2/compare/v8.3.0...v8.3.1)\n\n##### <!-- 2 -->🚜 Refactor\n\n- use `AexEncryption::new`\n([#&#8203;736](https://redirect.github.com/zip-rs/zip2/pull/736))\n- update tests to add big endian miri check\n([#&#8203;735](https://redirect.github.com/zip-rs/zip2/pull/735))\n\n##### <!-- 7 -->⚙️ Miscellaneous Tasks\n\n- cleanup repository files\n([#&#8203;743](https://redirect.github.com/zip-rs/zip2/pull/743))\n\n###\n[`v8.3.0`](https://redirect.github.com/zip-rs/zip2/blob/HEAD/CHANGELOG.md#830---2026-03-19)\n\n[Compare\nSource](https://redirect.github.com/zip-rs/zip2/compare/v8.2.0...v8.3.0)\n\n##### <!-- 0 -->🚀 Features\n\n- add must\\_use\n([#&#8203;727](https://redirect.github.com/zip-rs/zip2/pull/727))\n- improve and fix extended timestamp extra field parsing\n([#&#8203;713](https://redirect.github.com/zip-rs/zip2/pull/713))\n- add crc32 ignore option\n([#&#8203;710](https://redirect.github.com/zip-rs/zip2/pull/710))\n- path related code in single file\n([#&#8203;712](https://redirect.github.com/zip-rs/zip2/pull/712))\n\n##### <!-- 1 -->🐛 Bug Fixes\n\n- Malformed ZIP64 file output\n([#&#8203;715](https://redirect.github.com/zip-rs/zip2/pull/715))\n([#&#8203;717](https://redirect.github.com/zip-rs/zip2/pull/717))\n\n##### <!-- 2 -->🚜 Refactor\n\n- refactor some imports\n([#&#8203;734](https://redirect.github.com/zip-rs/zip2/pull/734))\n- move code to distinct file (datetime, FixedSizeBlock)\n([#&#8203;733](https://redirect.github.com/zip-rs/zip2/pull/733))\n- move stream code to `src/read/stream.rs`\n([#&#8203;731](https://redirect.github.com/zip-rs/zip2/pull/731))\n- remove zip64 extra field update\n([#&#8203;732](https://redirect.github.com/zip-rs/zip2/pull/732))\n- improve part of the code with clippy help\n([#&#8203;725](https://redirect.github.com/zip-rs/zip2/pull/725))\n- simplify code for unicode extra field and improve error message\n([#&#8203;724](https://redirect.github.com/zip-rs/zip2/pull/724))\n- reorganize code\n([#&#8203;714](https://redirect.github.com/zip-rs/zip2/pull/714))\n\n##### Deps\n\n- avoid pulling in `zeroize_derive`\n([#&#8203;720](https://redirect.github.com/zip-rs/zip2/pull/720))\n\n###\n[`v8.2.0`](https://redirect.github.com/zip-rs/zip2/blob/HEAD/CHANGELOG.md#820---2026-03-02)\n\n[Compare\nSource](https://redirect.github.com/zip-rs/zip2/compare/v8.1.0...v8.2.0)\n\n##### <!-- 0 -->🚀 Features\n\n- allow custom salt\n([#&#8203;680](https://redirect.github.com/zip-rs/zip2/pull/680))\n- Support compressing bzip2 when feature `bzip2-rs` is enabled, since\n`bzip2/bzip2-sys` now supports it\n([#&#8203;685](https://redirect.github.com/zip-rs/zip2/pull/685))\n- enforce clippy in CI\n([#&#8203;674](https://redirect.github.com/zip-rs/zip2/pull/674))\n\n##### <!-- 1 -->🐛 Bug Fixes\n\n- zip64 central header (issue 617)\n([#&#8203;629](https://redirect.github.com/zip-rs/zip2/pull/629))\n- allow aes password as bytes\n([#&#8203;686](https://redirect.github.com/zip-rs/zip2/pull/686))\n- handle extra field padding\n([#&#8203;682](https://redirect.github.com/zip-rs/zip2/pull/682))\n\n##### <!-- 2 -->🚜 Refactor\n\n- Simplify 2 type conversions in src/write.rs\n([#&#8203;687](https://redirect.github.com/zip-rs/zip2/pull/687))\n\n##### <!-- 4 -->⚡ Performance\n\n- AI tweaks for string type conversions in src/types.rs\n([#&#8203;670](https://redirect.github.com/zip-rs/zip2/pull/670))\n\n###\n[`v8.1.0`](https://redirect.github.com/zip-rs/zip2/blob/HEAD/CHANGELOG.md#810---2026-02-16)\n\n[Compare\nSource](https://redirect.github.com/zip-rs/zip2/compare/v8.0.0...v8.1.0)\n\n##### <!-- 0 -->🚀 Features\n\n- *(writer)* Allow getting underlying writer of ZipWriter\n([#&#8203;464](https://redirect.github.com/zip-rs/zip2/pull/464))\n- add system to FileOption, so byte-for-byte identical archives can be\ncreated across platforms\n([#&#8203;660](https://redirect.github.com/zip-rs/zip2/pull/660))\n\n##### <!-- 1 -->🐛 Bug Fixes\n\n- Bugs in extra-data length calculation in src/write.rs\n([#&#8203;662](https://redirect.github.com/zip-rs/zip2/pull/662))\n\n###\n[`v8.0.0`](https://redirect.github.com/zip-rs/zip2/blob/HEAD/CHANGELOG.md#800---2026-02-14)\n\n[Compare\nSource](https://redirect.github.com/zip-rs/zip2/compare/v7.4.0...v8.0.0)\n\n##### <!-- 0 -->🚀 Features\n\n- document zip flags as enum\n([#&#8203;639](https://redirect.github.com/zip-rs/zip2/pull/639))\n- Migrate to Rust 2024\n([#&#8203;650](https://redirect.github.com/zip-rs/zip2/pull/650))\n- \\[**breaking**] Remove deprecated methods of `DateTime`\n([#&#8203;597](https://redirect.github.com/zip-rs/zip2/pull/597))\n\n###\n[`v7.4.0`](https://redirect.github.com/zip-rs/zip2/blob/HEAD/CHANGELOG.md#740---2026-02-05)\n\n[Compare\nSource](https://redirect.github.com/zip-rs/zip2/compare/v7.3.0...v7.4.0)\n\n##### <!-- 0 -->🚀 Features\n\n- Increase MSRV to 1.88 and update dependencies\n([#&#8203;626](https://redirect.github.com/zip-rs/zip2/pull/626))\n\n###\n[`v7.3.0`](https://redirect.github.com/zip-rs/zip2/blob/HEAD/CHANGELOG.md#730---2026-02-04)\n\n[Compare\nSource](https://redirect.github.com/zip-rs/zip2/compare/v7.2.0...v7.3.0)\n\n##### <!-- 0 -->🚀 Features\n\n- cleanup the benchmarks and Cargo.toml\n([#&#8203;606](https://redirect.github.com/zip-rs/zip2/pull/606))\n- Add support for per-file comments\n([#&#8203;543](https://redirect.github.com/zip-rs/zip2/pull/543))\n\n##### <!-- 1 -->🐛 Bug Fixes\n\n- Document feature `unreserved` and make the mapping of extra fields\npublic ([#&#8203;616](https://redirect.github.com/zip-rs/zip2/pull/616))\n- Return an error if abort\\_file() fails when exceeding non-large-file\nlimit ([#&#8203;598](https://redirect.github.com/zip-rs/zip2/pull/598))\n\n##### <!-- 7 -->⚙️ Miscellaneous Tasks\n\n- Bump version to 7.3.0 (semver checks fail if it's still 7.3.0-pre1)\n\n###\n[`v7.2.0`](https://redirect.github.com/zip-rs/zip2/blob/HEAD/CHANGELOG.md#720---2026-01-20)\n\n[Compare\nSource](https://redirect.github.com/zip-rs/zip2/compare/v7.1.0...v7.2.0)\n\n##### <!-- 0 -->🚀 Features\n\n- add read\\_zipfile\\_from\\_stream\\_with\\_compressed\\_size\n([#&#8203;70](https://redirect.github.com/zip-rs/zip2/pull/70))\n- Allow choosing bzip2 rust backend\n([#&#8203;329](https://redirect.github.com/zip-rs/zip2/pull/329))\n\n##### <!-- 1 -->🐛 Bug Fixes\n\n- Need to include zip64 extra field in central directory (fix\n[#&#8203;353](https://redirect.github.com/zip-rs/zip2/issues/353))\n([#&#8203;360](https://redirect.github.com/zip-rs/zip2/pull/360))\n- Fails to extract file which might or might not be malformed\n([#&#8203;376](https://redirect.github.com/zip-rs/zip2/pull/376))\n([#&#8203;426](https://redirect.github.com/zip-rs/zip2/pull/426))\n- *(aes)* Allow AES encryption while streaming\n([#&#8203;463](https://redirect.github.com/zip-rs/zip2/pull/463))\n- Default \"platform\" field in zip files should be set to the local\nplatform, rather than always \"Unix\"\n([#&#8203;470](https://redirect.github.com/zip-rs/zip2/pull/470))\n([#&#8203;471](https://redirect.github.com/zip-rs/zip2/pull/471))\n\n##### <!-- 2 -->🚜 Refactor\n\n- Define cfg\\_if! and cfg\\_if\\_expr! internal macros\n([#&#8203;438](https://redirect.github.com/zip-rs/zip2/pull/438))\n\n##### <!-- 4 -->⚡ Performance\n\n- Change an assert to debug\\_assert when encrypting/decrypting AES, and\neliminate a fallible operation\n([#&#8203;521](https://redirect.github.com/zip-rs/zip2/pull/521))\n- eliminate a String clone per new file added to archive, and other\nrelated refactors\n([#&#8203;522](https://redirect.github.com/zip-rs/zip2/pull/522))\n\n###\n[`v7.1.0`](https://redirect.github.com/zip-rs/zip2/blob/HEAD/CHANGELOG.md#710---2026-01-14)\n\n[Compare\nSource](https://redirect.github.com/zip-rs/zip2/compare/v7.0.0...v7.1.0)\n\n##### <!-- 0 -->🚀 Features\n\n- display the underlying error in Display impl for `ZipError`\n([#&#8203;483](https://redirect.github.com/zip-rs/zip2/pull/483))\n- Enable creation of `ZipArchive` without reparsing\n([#&#8203;485](https://redirect.github.com/zip-rs/zip2/pull/485))\n\n##### <!-- 1 -->🐛 Bug Fixes\n\n- Return InvalidPassword rather than panic when AES key is the wrong\nlength ([#&#8203;457](https://redirect.github.com/zip-rs/zip2/pull/457))\n- bench with auto zip64 comment\n([#&#8203;505](https://redirect.github.com/zip-rs/zip2/pull/505))\n- add condition for `getrandom` dependency\n([#&#8203;504](https://redirect.github.com/zip-rs/zip2/pull/504))\n- *(zipcrypto)* Support streaming ZipCrypto encryption, don't store\nentire file in memory\n([#&#8203;462](https://redirect.github.com/zip-rs/zip2/pull/462))\n\n##### <!-- 2 -->🚜 Refactor\n\n- Clean up imports and move types\n([#&#8203;461](https://redirect.github.com/zip-rs/zip2/pull/461))\n- Replace handwritten `Ord` and `PartialOrd` for `DateTime`\n([#&#8203;484](https://redirect.github.com/zip-rs/zip2/pull/484))\n\n##### <!-- 7 -->⚙️ Miscellaneous Tasks\n\n- Lock `lzma-rust2` to at least 0.15.5\n([#&#8203;491](https://redirect.github.com/zip-rs/zip2/pull/491))\n\n###\n[`v7.0.0`](https://redirect.github.com/zip-rs/zip2/blob/HEAD/CHANGELOG.md#700---2025-12-05)\n\n[Compare\nSource](https://redirect.github.com/zip-rs/zip2/compare/v6.0.0...v7.0.0)\n\n##### <!-- 9 -->⚠️ Breaking Changes\n\n- Removed the following features: `getrandom`, `hmac`, `pbkdf2`, `sha1`,\n`zeroize`.\n- Removed `lzma-static` and `xz-static` feature flags, which were\ndeprecated synonyms of `lzma` and `xz`.\n([#&#8203;405](https://redirect.github.com/zip-rs/zip2/pull/405),\n[#&#8203;425](https://redirect.github.com/zip-rs/zip2/pull/425))\n\n##### <!-- 0 -->🚀 Features\n\n- *(`SimpleFileOptions`)* const DEFAULT implementation\n([#&#8203;474](https://redirect.github.com/zip-rs/zip2/pull/474))\n- ZipWriter `set_auto_large_file()` method to enable large-file data\ndescriptor when necessary\n([#&#8203;468](https://redirect.github.com/zip-rs/zip2/pull/468))\n\n##### <!-- 1 -->🐛 Bug Fixes\n\n- print previous error when failing to search another cde\n([#&#8203;460](https://redirect.github.com/zip-rs/zip2/pull/460))\n- cargo doc warnings\n([#&#8203;472](https://redirect.github.com/zip-rs/zip2/pull/472))\n- Write ZIP64 data descriptors when large\\_file option is true\n([#&#8203;467](https://redirect.github.com/zip-rs/zip2/pull/467))\n- Pin generic-array to an old version to work around\n[RustCrypto/traits#2036](https://redirect.github.com/RustCrypto/traits/issues/2036)\nuntil next RustCrypto & aes-crypto releases\n([#&#8203;458](https://redirect.github.com/zip-rs/zip2/pull/458))\n\n##### <!-- 7 -->⚙️ Miscellaneous Tasks\n\n- Revert version bump so that release-plz will trigger\n- expose more flate2 feature flags\n([#&#8203;476](https://redirect.github.com/zip-rs/zip2/pull/476))\n- Next release will be 7.0.0\n- release v6.0.0\n([#&#8203;442](https://redirect.github.com/zip-rs/zip2/pull/442))\n\n##### Deps\n\n- Bump lzma-rust2 to v0.15\n([#&#8203;465](https://redirect.github.com/zip-rs/zip2/pull/465))\n\n###\n[`v6.0.0`](https://redirect.github.com/zip-rs/zip2/blob/HEAD/CHANGELOG.md#700---2025-12-05)\n\n[Compare\nSource](https://redirect.github.com/zip-rs/zip2/compare/v5.1.1...v6.0.0)\n\n##### <!-- 9 -->⚠️ Breaking Changes\n\n- Removed the following features: `getrandom`, `hmac`, `pbkdf2`, `sha1`,\n`zeroize`.\n- Removed `lzma-static` and `xz-static` feature flags, which were\ndeprecated synonyms of `lzma` and `xz`.\n([#&#8203;405](https://redirect.github.com/zip-rs/zip2/pull/405),\n[#&#8203;425](https://redirect.github.com/zip-rs/zip2/pull/425))\n\n##### <!-- 0 -->🚀 Features\n\n- *(`SimpleFileOptions`)* const DEFAULT implementation\n([#&#8203;474](https://redirect.github.com/zip-rs/zip2/pull/474))\n- ZipWriter `set_auto_large_file()` method to enable large-file data\ndescriptor when necessary\n([#&#8203;468](https://redirect.github.com/zip-rs/zip2/pull/468))\n\n##### <!-- 1 -->🐛 Bug Fixes\n\n- print previous error when failing to search another cde\n([#&#8203;460](https://redirect.github.com/zip-rs/zip2/pull/460))\n- cargo doc warnings\n([#&#8203;472](https://redirect.github.com/zip-rs/zip2/pull/472))\n- Write ZIP64 data descriptors when large\\_file option is true\n([#&#8203;467](https://redirect.github.com/zip-rs/zip2/pull/467))\n- Pin generic-array to an old version to work around\n[RustCrypto/traits#2036](https://redirect.github.com/RustCrypto/traits/issues/2036)\nuntil next RustCrypto & aes-crypto releases\n([#&#8203;458](https://redirect.github.com/zip-rs/zip2/pull/458))\n\n##### <!-- 7 -->⚙️ Miscellaneous Tasks\n\n- Revert version bump so that release-plz will trigger\n- expose more flate2 feature flags\n([#&#8203;476](https://redirect.github.com/zip-rs/zip2/pull/476))\n- Next release will be 7.0.0\n- release v6.0.0\n([#&#8203;442](https://redirect.github.com/zip-rs/zip2/pull/442))\n\n##### Deps\n\n- Bump lzma-rust2 to v0.15\n([#&#8203;465](https://redirect.github.com/zip-rs/zip2/pull/465))\n\n###\n[`v5.1.1`](https://redirect.github.com/zip-rs/zip2/blob/HEAD/CHANGELOG.md#511---2025-09-11)\n\n[Compare\nSource](https://redirect.github.com/zip-rs/zip2/compare/v5.1.0...v5.1.1)\n\n##### <!-- 1 -->🐛 Bug Fixes\n\n- panic when reading empty extended-timestamp field\n([#&#8203;404](https://redirect.github.com/zip-rs/zip2/pull/404))\n([#&#8203;422](https://redirect.github.com/zip-rs/zip2/pull/422))\n- Restore original file timestamp when unzipping with `chrono`\n([#&#8203;46](https://redirect.github.com/zip-rs/zip2/pull/46))\n\n##### <!-- 7 -->⚙️ Miscellaneous Tasks\n\n- Configure Amazon Q rules\n([#&#8203;421](https://redirect.github.com/zip-rs/zip2/pull/421))\n\n###\n[`v5.1.0`](https://redirect.github.com/zip-rs/zip2/blob/HEAD/CHANGELOG.md#510---2025-09-10)\n\n[Compare\nSource](https://redirect.github.com/zip-rs/zip2/compare/v5.0.1...v5.1.0)\n\n##### <!-- 0 -->🚀 Features\n\n- Add legacy shrink/reduce/implode compression\n([#&#8203;303](https://redirect.github.com/zip-rs/zip2/pull/303))\n\n###\n[`v5.0.1`](https://redirect.github.com/zip-rs/zip2/blob/HEAD/CHANGELOG.md#501---2025-09-09)\n\n[Compare\nSource](https://redirect.github.com/zip-rs/zip2/compare/v5.0.0...v5.0.1)\n\n##### <!-- 1 -->🐛 Bug Fixes\n\n- AES metadata was not copied correctly in raw copy methods, which could\ncorrupt the copied file.\n([#&#8203;417](https://redirect.github.com/zip-rs/zip2/pull/417))\n\n###\n[`v5.0.0`](https://redirect.github.com/zip-rs/zip2/blob/HEAD/CHANGELOG.md#500---2025-09-05)\n\n[Compare\nSource](https://redirect.github.com/zip-rs/zip2/compare/v4.6.1...v5.0.0)\n\n##### <!-- 0 -->🚀 Features\n\n- Implement by\\_path\\*() methods on ZipArchive\n([#&#8203;382](https://redirect.github.com/zip-rs/zip2/pull/382))\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: (UTC)\n\n- Branch creation\n  - \"before 8am on Monday\"\n- Automerge\n  - At any time (no schedule defined)\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My4xNDEuMyIsInVwZGF0ZWRJblZlciI6IjQzLjE0MS4zIiwidGFyZ2V0QnJhbmNoIjoibWFpbiIsImxhYmVscyI6WyJkZXBlbmRlbmNpZXMiXX0=-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2026-04-28T03:20:25Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/9eab7837283aaa7f64ea77ae318fbb236d1f3794"
        },
        "date": 1777397510454,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.208980447923,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.44852784016108,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.454036458333334,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 39.4609375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 0,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 113609.42327568226,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002065,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 774740.9583812082,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 32839742.2251636,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.09824237337502,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.2462500967867,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.985416666666666,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.58984375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 0,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 164165.3545087858,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002307,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 879778.5998974914,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 47403556.983518794,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 97.061173765063,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 97.42262688403568,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 35.43567708333333,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 35.9296875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 0,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 113653.91409030712,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002333,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2176125.213732136,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 32873526.09436868,
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
            "value": 95.21347058522419,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.01671122163506,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 29.8578125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 30.921875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 0,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 244567.51796717144,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00388,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2710904.886608921,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 71240530.42746261,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Network Utilization"
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
          "id": "0ddaeb436473414388587dd60b58bfddcdf7e226",
          "message": "Add dfctl CLI and TUI for OTAP Dataflow Engine administration (#2756)\n\n# Change Summary\n\nAdds `dfctl`, an admin SDK-based command-line tool for operating local\nand remote OTAP Dataflow Engines.\n\nThe CLI supports engine, group, pipeline, telemetry, rollout, shutdown,\nreconfiguration, diagnosis, bundle, watch, shell completion, automation\nfriendly output, and an interactive TUI for operational workflows.\n\nMore details can be find here ->\nhttps://github.com/lquerel/otel-arrow/blob/962a01e30116433e448ed58b6d8b820e1bcdcd3a/rust/otap-dataflow/crates/enginectl/README.md\n\n## What issue does this PR close?\n\nN/A\n\n## How are these changes tested?\n\n- `cargo xtask check`\n\n## Are there any user-facing changes?\n\nYes, the new CLI by itself.\n\n<img width=\"1888\" height=\"1197\" alt=\"image\"\nsrc=\"https://github.com/user-attachments/assets/6ee53571-b1c5-47b2-bdcd-6d06999ea7d5\"\n/>\n\n<img width=\"1888\" height=\"1197\" alt=\"image\"\nsrc=\"https://github.com/user-attachments/assets/21e6273d-9fc5-423d-b4d8-1472d9f40059\"\n/>",
          "timestamp": "2026-04-29T00:24:10Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/0ddaeb436473414388587dd60b58bfddcdf7e226"
        },
        "date": 1777426664947,
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
            "value": 100.15647694986558,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.33376384049555,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.433723958333335,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.98828125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 0,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 161081.08789165603,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002078,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 872316.3272061608,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 47199907.952280484,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 93.47996716991516,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 97.15616744186046,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 35.12096354166667,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 35.94921875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 0,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 111462.78590400411,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002089,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2112836.32507499,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 32009301.685759615,
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
            "value": 95.75730073955943,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 95.99751263116984,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 30.23671875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 31.49609375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 0,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 243039.37107817153,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002624,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2741401.0964374454,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 71413665.08618191,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 23.87791375754374,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.241549186677,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.792057291666666,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 43.5,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 0,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 27187.349604400148,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002171,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 230334.01021538323,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 8303507.88835993,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Network Utilization"
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
          "id": "dccbcb2de29f5c993d5dec24a558e8db3ad74322",
          "message": "[query-engine] Improve value equality logic (#2775)\n\n# Changes\n\n* Improve the value equality helper logic in expressions\n\n# Details\n\nThe way things were coded, left-side of equality comparison determined\nthe type. So...\n\n* `int(18) == string('value')` results in an error that string on\nright-side could not be converted to int (evaluates as `int(18) ==\ntoint(string('value'))`)\n* `string('value') == int(18)` results in `false` (evaluates as\n`string('value') == tostring(int(18))`)\n\nThis PR adjusts the logic so that if either side of a comparison is a\nstring we will apply string logic. So...\n\n* `int(18) == string('value')` results in `false` (evaluates as\n`tostring(int(18)) == string('value')`)\n* `string('value') == int(18)` results in `false` (evaluates as\n`string('value') == tostring(int(18))`)",
          "timestamp": "2026-04-29T16:33:32Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/dccbcb2de29f5c993d5dec24a558e8db3ad74322"
        },
        "date": 1777482964170,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.207292377861,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.43258709926613,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.536197916666666,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.453125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 0,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 115152.51349934646,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002164,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 789030.7430448929,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 32715580.308868513,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 95.89287040270516,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.19802579748205,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 30.710416666666667,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 31.515625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 0,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 245636.22799548373,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003364,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2750621.645378469,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 71579278.45729692,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 72.16915756733964,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 73.38468631546375,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 35.719140625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 36.90234375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 0,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 76370.34979833454,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002344,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 450560.33527235425,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 22385275.871510375,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 97.08071575455998,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 97.48733297372274,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 36.03958333333333,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 36.671875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 0,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 113887.72615134182,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002076,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2181535.536002236,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 32986001.830513228,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Max Jacinto",
            "username": "luckymachi",
            "email": "77021922+luckymachi@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "775794600fb4ba7bea406b4e0bb08cd598b10cda",
          "message": "Isolation of setup-protoc jobs from RUST-CI (#2772)\n\n# Change Summary\n\nAs mentioned in issue\n[2768](https://github.com/open-telemetry/otel-arrow/issues/2768),\n`setup-protoc` jobs are dropped in favour of a targeted `compile_proto`\njob.\n\n## What issue does this PR close?\n\nThis issue closes issue\n[2768](https://github.com/open-telemetry/otel-arrow/issues/2768)\n\n---------\n\nCo-authored-by: Drew Relmas <drewrelmas@gmail.com>",
          "timestamp": "2026-04-29T20:45:20Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/775794600fb4ba7bea406b4e0bb08cd598b10cda"
        },
        "date": 1777520199085,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.97284452834874,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 97.23456208339792,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 35.390234375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 35.86328125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 0,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 115336.33043911168,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002776,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2190087.28867993,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 33101913.85560192,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.1788733676917,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.37215483272813,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.219140625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.0859375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 0,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 165289.14049996927,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002127,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 896913.5237943807,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 47691728.80407968,
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
            "value": 100.2286030002687,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.50165208639777,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.91953125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.109375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 0,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 113738.3091970161,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.005288,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 793518.1670690401,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 32876492.01695069,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 94.51383072751463,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.02367447595562,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 30.485026041666668,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 31.2421875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 0,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 244584.2976880884,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003852,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2764557.307881717,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 71891603.23801808,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Network Utilization"
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
          "id": "524b78de5762aaebe99e31d67d3a2351d4d9e303",
          "message": "Make pipeline perf test a required CI check (#2779)\n\n## Summary\n\nMake the pipeline performance test a required CI check so that PRs which\nbreak the perf test are caught before merge.\n\n> **Dependency**: #2780 must be merged first (it fixes the currently\nbroken perf test).\n\n#2774 is an example of the kind of breakage this prevents — a route\nrename broke the perf test but the PR still merged because the perf test\nwas not a required check.\n\n### Changes\n\n- **rust-ci.yml**: Add `pipeline_perf_test` job (runs on\n`ubuntu-latest`) and include it in `rust-required-status-check`\naggregator\n- **pipeline-perf-on-label.yaml**: Simplify to only run on dedicated\nOracle bare-metal hardware when `pipelineperf` label is present — the\nbasic validation path is removed since `rust-ci.yml` now covers it\n\n### Motivation\n\nThe pipeline perf test has been broken by merged PRs several times\nbecause it was not a required check. This change ensures that if a PR\nbreaks the perf test (e.g. build failures, config issues, test\ninfrastructure breakage), it is caught before merge.\n\n---------\n\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-04-30T12:25:55Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/524b78de5762aaebe99e31d67d3a2351d4d9e303"
        },
        "date": 1777571643425,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.17977942001717,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.29119305856833,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.258723958333334,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 16.8203125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 0,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 113710.83628821124,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002197,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 791952.3792065541,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 32855300.175615527,
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
            "value": 97.0463963809166,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 97.36890674495866,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 36.227213541666664,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 36.984375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 0,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 115329.29296376984,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002102,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2191155.601414493,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 33110118.15681137,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.1721475372253,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.31302408425617,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.119140625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.19140625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 0,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 165403.9682686238,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002188,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 901412.6538005346,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 47739966.20771982,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 89.99127509178788,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 90.28957397366383,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 28.736848958333333,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 29.4375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 0,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 237954.75703107726,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003003,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2645774.170643592,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 69302054.30770634,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Network Utilization"
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
          "id": "993fa369f3ef1c12b381f01f758302e8243f211a",
          "message": "feat(engine): Extension System - Capability Registry & Resolver (#2732)\n\n# Extension system — Phase 1 (capabilities, registry, builder, proc\nmacro)\n\nImplements the Phase-1 extension/capability system for the OTAP dataflow\nengine. Extensions are first-class config siblings of nodes; nodes\nexplicitly bind to extension instances via named capabilities, and\nreceive typed handles resolved once at factory time — no hot-path\nregistry lookups.\n\nTracking docs:\n\n[`docs/extension-system-architecture.md`](rust/otap-dataflow/docs/extension-system-architecture.md)\n(rewritten in this PR).\n\n## What's in this PR\n\n### `#[capability]` proc macro (`engine-macros`)\n\n- New `capability.rs` expansion: from a single `#[capability] trait\n  Foo { ... }` source it generates `local::Foo` (`!Send`-friendly) and\n  `shared::Foo` (`Send + Clone`) trait variants plus a `SharedAsLocal`\n  adapter and an `ExtensionCapability` impl. The dual variants are\n  derived from one source, so authors can't accidentally let local\n  and shared semantics diverge.\n- New `pipeline_factory.rs` expansion to build the static\n  `PipelineFactory` registry used by `main.rs`.\n- All emitted paths use fully-qualified `::std::...` /\n  `::async_trait::...` / `::otap_df_engine::...` so generated code is\n  hygienic in any caller crate.\n\n### Capability registry (`engine::capability`)\n\n- `CapabilityRegistry`: typed-keyed (`(extension_name, TypeId)`)\n  storage with **typestate-enforced** single `.shared()` / `.local()`\n  registration per builder — duplicates are unrepresentable rather\n  than runtime errors.\n- Two execution models: native local (`Rc<dyn Local>`, lock-free) and\n  native shared (`Box<dyn Shared>`, `Send + Clone`). A shared-only\n  extension serves local consumers transparently via the\n  `SharedAsLocal` adapter generated by the proc macro.\n- Two **instance policies** chosen at build time, invisible to\n  consumers: `.cloned()` (clone a stored prototype) and\n  `.constructed()` (per-consumer construction via a closure;\n  Passive-only — `Active + Constructed` is statically rejected).\n- `resolve_bindings`: walks a node's `capabilities:` declaration and\n  produces a per-node `Capabilities` bundle with all bindings\n  resolved, surfacing config errors (unknown extension, unknown\n  capability, capability not provided by bound extension, multiple\n  bindings for the same capability).\n- `Capabilities`: per-node consumer API with `require_local`,\n  `require_shared`, `optional_local`, `optional_shared`. Instances\n  are minted lazily at the call site, not at resolution time.\n- `ConsumedTracker`: cross-node, per-(capability, extension)\n  consumption flags driving `drop_local()` / `drop_shared()` cleanup\n  for extensions no node ever claimed.\n\n### One-shot consumption contract\n\nA binding is claimable **at most once per node**, regardless of\nexecution model. The guard is the `Cell<Option<_>>::take()` on each\nresolved entry's `produce` closure — no auxiliary flag.\n\n- Same accessor twice → `CapabilityAlreadyConsumed`.\n- Different accessors on a SharedAsLocal-fallback binding share one\n  underlying entry, so claiming either side consumes the other\n  naturally.\n- Different accessors on a native-dual binding (extension registered\n  both native local **and** native shared) take and drop the\n  alternative entry's `produce` closure on success, so the\n  per-binding contract holds uniformly. The cross-node tracker is\n  only flipped by actual consumption, not by invalidation, so\n  `drop_*` cleanup remains correct.\n\n### Documentation\n\n- `docs/extension-system-architecture.md`: rewritten to describe the\n  capability-based design, the local/shared duality, instance\n  policies, Active vs Passive lifecycle, and the typestate builder.\n\n## Tests\n\nNew, focused unit tests cover:\n\n- Registry: typestate single-registration, duplicate rejection,\n  `SharedAsLocal` adapter freshness per node, double-`Box` envelope\n  for shared `produce`.\n- `resolve_bindings`: every error path (unknown extension / unknown\n  capability / capability not provided / wrong extension), local-only\n  and shared-only binding shapes, fallback path, native-dual path.\n- One-shot contract: second-call rejection on each of `require_local`,\n  `require_shared`, `optional_local`, `optional_shared`; fallback\n  cross-side rejection; native-dual cross-side rejection (both\n  directions).\n- `ConsumedTracker`: per-extension consumption flags, with the\n  invariant that mere invalidation does not flip a bucket.\n- Proc-macro end-to-end: `local-only`, `shared-only`, and `dual`\n  forms of `extension_capabilities!` against the registry.\n\n## Validation\n\n```text\ncargo xtask check\n✅ Cargo workspace structure complies with project policies.\n✅ Formatting completed successfully.\n✅ Clippy linting passed without warnings.\n✅ All tests passed successfully.\n```\n\n---------\n\nCo-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>",
          "timestamp": "2026-05-01T02:10:49Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/993fa369f3ef1c12b381f01f758302e8243f211a"
        },
        "date": 1777605299125,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.9422924644167,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 97.25600433805872,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 35.45677083333333,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 36.24609375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 0,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 114329.2193869224,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002159,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2191035.2947478523,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 33085188.317467354,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.22160424359492,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.3895059625213,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.923958333333335,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.3515625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 0,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 115661.09836561514,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002024,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 791116.9053286334,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 32927064.902282614,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 94.91769939073707,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.0921933085502,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 30.41796875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 31.30078125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 0,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 246330.314944653,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002765,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2755150.4136172556,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 72305185.31397443,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 71.57866759996975,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 73.07390798355186,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 35.04088541666667,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 36.015625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 0,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 75727.1691417652,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004884,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 447073.3498449825,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 22286093.628479507,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Cijo Thomas",
            "username": "cijothomas",
            "email": "cithomas@microsoft.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "ce3582596886edf41cb83c87829be8cd8d15fcce",
          "message": "fix(perf-test): add missing /api/v1 prefix to idle-state template endpoints (#2798)\n\nThe idle-state-template.yaml.j2 was using /telemetry/metrics instead of\n/api/v1/telemetry/metrics for both the Prometheus scraping endpoint and\nthe ready-check URL. This caused 404 errors during idle state\nbenchmarks.\n\nAll other test configs already had the correct /api/v1 prefix.\n\n# Change Summary\n\n<!--\nReplace with a brief summary of the change in this PR\n-->\n\n## What issue does this PR close?\n\n<!--\nWe highly recommend correlation of every PR to an issue\n-->\n\n* Closes #NNN\n\n## How are these changes tested?\n\n## Are there any user-facing changes?\n\n <!-- If yes, provide further info below -->",
          "timestamp": "2026-05-01T16:38:12Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/ce3582596886edf41cb83c87829be8cd8d15fcce"
        },
        "date": 1777658960293,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 95.20773597543938,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 95.4706901092092,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 29.953125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 30.703125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 0,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 248953.37255657217,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003204,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2775592.932447485,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 72556340.21521777,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.22218190199126,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.42905645724936,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.044010416666666,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.30859375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 0,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 115984.02066083175,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002231,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 795295.5035781078,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 33016003.751961682,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.17501810106604,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.31249456352904,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.502734375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.859375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 0,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 163380.68965643673,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002195,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 878318.8693886377,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 47157913.382388115,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 93.19652844841619,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 93.45481205343275,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 34.805729166666666,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 35.89453125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 0,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 111379.50002220756,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002065,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2129015.8252323833,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 32270105.94241816,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Gyan ranjan",
            "username": "gyanranjanpanda",
            "email": "213113461+gyanranjanpanda@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "f018901f3a2ba93bee9d45f6afb1204f90679863",
          "message": "Fix duplicate attribute keys in transform_attributes (#2423)\n\n# Fix Duplicate Attribute Keys in `transform_attributes`\n\n## Changes Made\nThis PR resolves issue #1650 by ensuring that dictionary keys are\ndeduplicated when transformations such as `rename` are applied, as\nrequired by the OpenTelemetry specification (\"Exported maps MUST contain\nonly unique keys by default\").\n\nTo accomplish this while maintaining strict performance requirements, we\nreplaced the previous `RowConverter` deduplication strategy with a new\nhigh-performance, proactive pre-filter:\n- We injected `filter_rename_collisions` into\n`transform_attributes_impl` inside\n`otap-dataflow/crates/pdata/src/otap/transform.rs`.\n- Before a rename is processed, this function reads the `parent_id`s and\ntarget keys. It uses the `IdBitmap` type to find any existing target\nkeys whose `parent_id` maps back to an old key that will be renamed.\n- It proactively strips those collision rows from the batch via\n`arrow::compute::filter_record_batch` *before* the actual transform\nhappens.\n\n## Testing\n- Extended the `AttributesProcessor` unit tests\n(`test_rename_removes_duplicate_keys`) to explicitly verify that\nrenaming an attribute resulting in a collision automatically discards\nduplicate keys.\n- Extended the `AttributesTransformPipelineStage` in `query-engine`\ntests with a parallel case ensuring OPL/KQL query pipelines\n(`project-rename`) properly drop duplicates when resolving duplicates.\n- Refactored `otap_df_pdata` `transform.rs` tests to properly expect\ndeduplicated keys using this plan-based method.\n- Validated logic with `cargo test --workspace --all-features`.\n\n## Validation Results\nAll tests pass. OTel semantic rules surrounding unique mapped keys map\ncleanly through down/upstream processors. The `IdBitmap` intersection\napproach completely resolves the multi-thousand percent `RowConverter`\nperformance regressions, dropping collision resolution overhead to\nessentially zero through efficient bitmap operations.\n\n---------\n\nSigned-off-by: Gyanranjan Panda <gyanranjanpanda438@gmail.com>\nCo-authored-by: Gyan Ranjan Panda <gyanranjanpanda@users.noreply.github.com>\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-05-01T20:08:07Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/f018901f3a2ba93bee9d45f6afb1204f90679863"
        },
        "date": 1777690078372,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.2316112566666,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.38429489661583,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.8453125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.09375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 0,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 115952.57740577015,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002116,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 795206.6181748023,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 33003927.9969912,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 71.28309508504859,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 72.63188092851084,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 35.832421875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 36.96484375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 0,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 75190.92445541824,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.007242,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 443358.2999020614,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 22145745.82508934,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.94035273328346,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 97.48088930203734,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 35.66002604166667,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 36.44140625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 0,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 115745.81385973208,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00217,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2201072.2974285656,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 33248370.79306773,
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
            "value": 95.80604979370139,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.08401084850833,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 29.612239583333334,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 30.8828125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 0,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 249828.24256243298,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003224,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2799678.9530026116,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 72686015.76197319,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Network Utilization"
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
          "id": "9aa767ee7b26712bbab69e4ecab5db2b22f80f32",
          "message": "Update github workflow dependencies (#2802)\n\n> ℹ️ **Note**\n> \n> This PR body was truncated due to platform limits.\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n|---|---|---|---|\n|\n[DavidAnson/markdownlint-cli2-action](https://redirect.github.com/DavidAnson/markdownlint-cli2-action)\n| action | minor | `v23.0.0` → `v23.1.0` |\n|\n[EmbarkStudios/cargo-deny-action](https://redirect.github.com/EmbarkStudios/cargo-deny-action)\n| action | patch | `v2.0.15` → `v2.0.17` |\n| [Swatinem/rust-cache](https://redirect.github.com/Swatinem/rust-cache)\n| action | minor | `v2` → `v2.9.1` |\n|\n[actions/create-github-app-token](https://redirect.github.com/actions/create-github-app-token)\n| action | minor | `v3.0.0` → `v3.1.1` |\n| [actions/setup-node](https://redirect.github.com/actions/setup-node) |\naction | minor | `v6.3.0` → `v6.4.0` |\n|\n[actions/upload-artifact](https://redirect.github.com/actions/upload-artifact)\n| action | patch | `v7.0.0` → `v7.0.1` |\n|\n[github/codeql-action](https://redirect.github.com/github/codeql-action)\n| action | patch | `v4.35.1` → `v4.35.3` |\n| [go](https://redirect.github.com/actions/go-versions) | uses-with |\npatch | `1.26.1` → `1.26.2` |\n|\n[step-security/harden-runner](https://redirect.github.com/step-security/harden-runner)\n| action | minor | `v2.16.1` → `v2.19.0` |\n|\n[taiki-e/install-action](https://redirect.github.com/taiki-e/install-action)\n| action | minor | `v2.71.2` → `v2.75.28` |\n\n---\n\n### Release Notes\n\n<details>\n<summary>DavidAnson/markdownlint-cli2-action\n(DavidAnson/markdownlint-cli2-action)</summary>\n\n###\n[`v23.1.0`](https://redirect.github.com/DavidAnson/markdownlint-cli2-action/releases/tag/v23.1.0):\nUpdate markdownlint-cli2 version (markdownlint-cli2 v0.22.1,\nmarkdownlint v0.40.0).\n\n[Compare\nSource](https://redirect.github.com/DavidAnson/markdownlint-cli2-action/compare/v23.0.0...v23.1.0)\n\n</details>\n\n<details>\n<summary>EmbarkStudios/cargo-deny-action\n(EmbarkStudios/cargo-deny-action)</summary>\n\n###\n[`v2.0.17`](https://redirect.github.com/EmbarkStudios/cargo-deny-action/releases/tag/v2.0.17):\nRelease 2.0.17 - cargo-deny 0.19.2\n\n[Compare\nSource](https://redirect.github.com/EmbarkStudios/cargo-deny-action/compare/v2.0.16...v2.0.17)\n\n##### Fixed\n\n-\n[PR#845](https://redirect.github.com/EmbarkStudios/cargo-deny/pull/845)\nfixed structural issues with SARIF output, resolving\n[#&#8203;818](https://redirect.github.com/EmbarkStudios/cargo-deny/issues/818).\nThanks\n[@&#8203;KyleChamberlin](https://redirect.github.com/KyleChamberlin)!\n\n###\n[`v2.0.16`](https://redirect.github.com/EmbarkStudios/cargo-deny-action/releases/tag/v2.0.16):\nRelease 2.0.16 - cargo-deny 0.19.1\n\n[Compare\nSource](https://redirect.github.com/EmbarkStudios/cargo-deny-action/compare/v2.0.15...v2.0.16)\n\n##### Fixed\n\n-\n[PR#833](https://redirect.github.com/EmbarkStudios/cargo-deny/pull/833)\nfixed an issue where the maximum advisory database staleness was over 14\nyears instead of the intended 90 days.\n-\n[PR#839](https://redirect.github.com/EmbarkStudios/cargo-deny/pull/839)\nfixed an issue where unsound advisories would appear for transitive\ndependencies despite requesting them only for workspace dependencies,\nresolving\n[#&#8203;829](https://redirect.github.com/EmbarkStudios/cargo-deny/issues/829).\n-\n[PR#840](https://redirect.github.com/EmbarkStudios/cargo-deny/pull/840)\nresolved\n[#&#8203;797](https://redirect.github.com/EmbarkStudios/cargo-deny/issues/797)\nby passing `--filter-platform` when collecting cargo metadata if only a\nsingle target was requested either in the config or via the command\nline.\n-\n[PR#841](https://redirect.github.com/EmbarkStudios/cargo-deny/pull/841)\nfixed an issue where `--frozen` would not disable fetching of the\nadvisory DB, resolving\n[#&#8203;759](https://redirect.github.com/EmbarkStudios/cargo-deny/issues/759).\n-\n[PR#842](https://redirect.github.com/EmbarkStudios/cargo-deny/pull/842)\nand\n[PR#844](https://redirect.github.com/EmbarkStudios/cargo-deny/pull/844)\nupdated crates. Notably `krates` was updated to resolve two issues with\ncrates being pruned from the graph used when running checks. Resolving\nthese two issues may mean that updating cargo-deny may highlight issues\nthat were previously hidden.\n-\n[EmbarkStudios/krates#106](https://redirect.github.com/EmbarkStudios/krates/issues/106)\nwould fail to pull in crates brought in via a feature if that crate had\nits `lib` target renamed by the package author.\n-\n[EmbarkStudios/krates#109](https://redirect.github.com/EmbarkStudios/krates/issues/109)\nwould fail to bring in optional dependencies if they were brought in by\na weak feature in a crate *also* brought in by a weak feature.\n\n##### Changed\n\n-\n[PR#830](https://redirect.github.com/EmbarkStudios/cargo-deny/pull/830)\nremoved `gix` in favor of shelling out to `git`. This massively improves\nbuild times and eases maintenance as `gix` bumps minor versions quite\nfrequently. If cargo-deny is used in an environment that for some reason\nallows internet access but doesn't have `git` available, the advisory\ndatabase would need to be updated before calling cargo-deny.\n-\n[PR#838](https://redirect.github.com/EmbarkStudios/cargo-deny/pull/838)\nremoved `rustsec` in favor of manually implemented advisory parsing and\nchecking, with a nightly cron job that checks that the implementation\nexactly matches rustsec on the official rustsec advisory db.\n\n</details>\n\n<details>\n<summary>Swatinem/rust-cache (Swatinem/rust-cache)</summary>\n\n###\n[`v2.9.1`](https://redirect.github.com/Swatinem/rust-cache/releases/tag/v2.9.1)\n\n[Compare\nSource](https://redirect.github.com/Swatinem/rust-cache/compare/v2.9.0...v2.9.1)\n\nFix regression in hash calculation\n\n**Full Changelog**:\n<https://github.com/Swatinem/rust-cache/compare/v2.9.0...v2.9.1>\n\n###\n[`v2.9.0`](https://redirect.github.com/Swatinem/rust-cache/releases/tag/v2.9.0)\n\n[Compare\nSource](https://redirect.github.com/Swatinem/rust-cache/compare/v2.8.2...v2.9.0)\n\n##### What's Changed\n\n- Add support for running rust-cache commands from within a Nix shell by\n[@&#8203;marc0246](https://redirect.github.com/marc0246) in\n[#&#8203;290](https://redirect.github.com/Swatinem/rust-cache/pull/290)\n- Bump taiki-e/install-action from 2.62.57 to 2.62.60 in the actions\ngroup by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;291](https://redirect.github.com/Swatinem/rust-cache/pull/291)\n- Bump the actions group across 1 directory with 5 updates by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;296](https://redirect.github.com/Swatinem/rust-cache/pull/296)\n- Bump the prd-major group with 3 updates by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;294](https://redirect.github.com/Swatinem/rust-cache/pull/294)\n- Bump [@&#8203;types/node](https://redirect.github.com/types/node) from\n24.10.1 to 25.0.2 in the dev-major group by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;295](https://redirect.github.com/Swatinem/rust-cache/pull/295)\n- Consider all installed toolchains in cache key by\n[@&#8203;tamird](https://redirect.github.com/tamird) in\n[#&#8203;293](https://redirect.github.com/Swatinem/rust-cache/pull/293)\n- Compare case-insenitively for full cache key match by\n[@&#8203;kbriggs](https://redirect.github.com/kbriggs) in\n[#&#8203;303](https://redirect.github.com/Swatinem/rust-cache/pull/303)\n- Migrate to `node24` runner by\n[@&#8203;rhysd](https://redirect.github.com/rhysd) in\n[#&#8203;314](https://redirect.github.com/Swatinem/rust-cache/pull/314)\n- Bump the actions group across 1 directory with 7 updates by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;312](https://redirect.github.com/Swatinem/rust-cache/pull/312)\n- Bump the prd-minor group across 1 directory with 2 updates by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;307](https://redirect.github.com/Swatinem/rust-cache/pull/307)\n- Bump [@&#8203;types/node](https://redirect.github.com/types/node) from\n25.0.2 to 25.2.2 in the dev-minor group by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;309](https://redirect.github.com/Swatinem/rust-cache/pull/309)\n\n##### New Contributors\n\n- [@&#8203;marc0246](https://redirect.github.com/marc0246) made their\nfirst contribution in\n[#&#8203;290](https://redirect.github.com/Swatinem/rust-cache/pull/290)\n- [@&#8203;tamird](https://redirect.github.com/tamird) made their first\ncontribution in\n[#&#8203;293](https://redirect.github.com/Swatinem/rust-cache/pull/293)\n- [@&#8203;kbriggs](https://redirect.github.com/kbriggs) made their\nfirst contribution in\n[#&#8203;303](https://redirect.github.com/Swatinem/rust-cache/pull/303)\n\n**Full Changelog**:\n<https://github.com/Swatinem/rust-cache/compare/v2.8.2...v2.9.0>\n\n###\n[`v2.8.2`](https://redirect.github.com/Swatinem/rust-cache/releases/tag/v2.8.2)\n\n[Compare\nSource](https://redirect.github.com/Swatinem/rust-cache/compare/v2.8.1...v2.8.2)\n\n##### What's Changed\n\n- ci: address lint findings, add zizmor workflow by\n[@&#8203;woodruffw](https://redirect.github.com/woodruffw) in\n[#&#8203;262](https://redirect.github.com/Swatinem/rust-cache/pull/262)\n- feat: Implement ability to disable adding job ID + rust environment\nhashes to cache names by\n[@&#8203;Ryan-Brice](https://redirect.github.com/Ryan-Brice) in\n[#&#8203;279](https://redirect.github.com/Swatinem/rust-cache/pull/279)\n- Don't overwrite env for cargo-metadata call by\n[@&#8203;MaeIsBad](https://redirect.github.com/MaeIsBad) in\n[#&#8203;285](https://redirect.github.com/Swatinem/rust-cache/pull/285)\n\n##### New Contributors\n\n- [@&#8203;woodruffw](https://redirect.github.com/woodruffw) made their\nfirst contribution in\n[#&#8203;262](https://redirect.github.com/Swatinem/rust-cache/pull/262)\n- [@&#8203;Ryan-Brice](https://redirect.github.com/Ryan-Brice) made\ntheir first contribution in\n[#&#8203;279](https://redirect.github.com/Swatinem/rust-cache/pull/279)\n- [@&#8203;MaeIsBad](https://redirect.github.com/MaeIsBad) made their\nfirst contribution in\n[#&#8203;285](https://redirect.github.com/Swatinem/rust-cache/pull/285)\n\n**Full Changelog**:\n<https://github.com/Swatinem/rust-cache/compare/v2.8.1...v2.8.2>\n\n###\n[`v2.8.1`](https://redirect.github.com/Swatinem/rust-cache/releases/tag/v2.8.1)\n\n[Compare\nSource](https://redirect.github.com/Swatinem/rust-cache/compare/v2.8.0...v2.8.1)\n\n##### What's Changed\n\n- Set empty `CARGO_ENCODED_RUSTFLAGS` in workspace metadata retrieval by\n[@&#8203;ark0f](https://redirect.github.com/ark0f) in\n[#&#8203;249](https://redirect.github.com/Swatinem/rust-cache/pull/249)\n- chore(deps): update dependencies by\n[@&#8203;reneleonhardt](https://redirect.github.com/reneleonhardt) in\n[#&#8203;251](https://redirect.github.com/Swatinem/rust-cache/pull/251)\n- chore: fix dependabot groups by\n[@&#8203;reneleonhardt](https://redirect.github.com/reneleonhardt) in\n[#&#8203;253](https://redirect.github.com/Swatinem/rust-cache/pull/253)\n- Bump the prd-patch group with 2 updates by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;254](https://redirect.github.com/Swatinem/rust-cache/pull/254)\n- chore(dependabot): regenerate and commit dist/ by\n[@&#8203;reneleonhardt](https://redirect.github.com/reneleonhardt) in\n[#&#8203;257](https://redirect.github.com/Swatinem/rust-cache/pull/257)\n- Bump [@&#8203;types/node](https://redirect.github.com/types/node) from\n22.16.3 to 24.2.1 in the dev-major group by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;255](https://redirect.github.com/Swatinem/rust-cache/pull/255)\n- Bump typescript from 5.8.3 to 5.9.2 in the dev-minor group by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;256](https://redirect.github.com/Swatinem/rust-cache/pull/256)\n- Bump actions/setup-node from 4 to 5 in the actions group by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;259](https://redirect.github.com/Swatinem/rust-cache/pull/259)\n- Update README.md by\n[@&#8203;Propfend](https://redirect.github.com/Propfend) in\n[#&#8203;234](https://redirect.github.com/Swatinem/rust-cache/pull/234)\n- Bump [@&#8203;types/node](https://redirect.github.com/types/node) from\n24.2.1 to 24.3.0 in the dev-minor group by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;258](https://redirect.github.com/Swatinem/rust-cache/pull/258)\n\n##### New Contributors\n\n- [@&#8203;ark0f](https://redirect.github.com/ark0f) made their first\ncontribution in\n[#&#8203;249](https://redirect.github.com/Swatinem/rust-cache/pull/249)\n- [@&#8203;reneleonhardt](https://redirect.github.com/reneleonhardt)\nmade their first contribution in\n[#&#8203;251](https://redirect.github.com/Swatinem/rust-cache/pull/251)\n- [@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot]\nmade their first contribution in\n[#&#8203;254](https://redirect.github.com/Swatinem/rust-cache/pull/254)\n- [@&#8203;Propfend](https://redirect.github.com/Propfend) made their\nfirst contribution in\n[#&#8203;234](https://redirect.github.com/Swatinem/rust-cache/pull/234)\n\n**Full Changelog**:\n<https://github.com/Swatinem/rust-cache/compare/v2...v2.8.1>\n\n###\n[`v2.8.0`](https://redirect.github.com/Swatinem/rust-cache/releases/tag/v2.8.0)\n\n[Compare\nSource](https://redirect.github.com/Swatinem/rust-cache/compare/v2.7.8...v2.8.0)\n\n##### What's Changed\n\n- Add cache-workspace-crates feature by\n[@&#8203;jbransen](https://redirect.github.com/jbransen) in\n[#&#8203;246](https://redirect.github.com/Swatinem/rust-cache/pull/246)\n- Feat: support warpbuild cache provider by\n[@&#8203;stegaBOB](https://redirect.github.com/stegaBOB) in\n[#&#8203;247](https://redirect.github.com/Swatinem/rust-cache/pull/247)\n\n##### New Contributors\n\n- [@&#8203;jbransen](https://redirect.github.com/jbransen) made their\nfirst contribution in\n[#&#8203;246](https://redirect.github.com/Swatinem/rust-cache/pull/246)\n- [@&#8203;stegaBOB](https://redirect.github.com/stegaBOB) made their\nfirst contribution in\n[#&#8203;247](https://redirect.github.com/Swatinem/rust-cache/pull/247)\n\n**Full Changelog**:\n<https://github.com/Swatinem/rust-cache/compare/v2.7.8...v2.8.0>\n\n###\n[`v2.7.8`](https://redirect.github.com/Swatinem/rust-cache/releases/tag/v2.7.8)\n\n[Compare\nSource](https://redirect.github.com/Swatinem/rust-cache/compare/v2.7.7...v2.7.8)\n\n##### What's Changed\n\n- Include CPU arch in the cache key for arm64 Linux runners by\n[@&#8203;rhysd](https://redirect.github.com/rhysd) in\n[#&#8203;228](https://redirect.github.com/Swatinem/rust-cache/pull/228)\n\n**Full Changelog**:\n<https://github.com/Swatinem/rust-cache/compare/v2.7.7...v2.7.8>\n\n###\n[`v2.7.7`](https://redirect.github.com/Swatinem/rust-cache/releases/tag/v2.7.7)\n\n[Compare\nSource](https://redirect.github.com/Swatinem/rust-cache/compare/v2.7.6...v2.7.7)\n\n**Full Changelog**:\n<https://github.com/Swatinem/rust-cache/compare/v2.7.6...v2.7.7>\n\n###\n[`v2.7.6`](https://redirect.github.com/Swatinem/rust-cache/releases/tag/v2.7.6)\n\n[Compare\nSource](https://redirect.github.com/Swatinem/rust-cache/compare/v2.7.5...v2.7.6)\n\n##### What's Changed\n\n- Updated artifact upload action to v4 by\n[@&#8203;guylamar2006](https://redirect.github.com/guylamar2006) in\n[#&#8203;212](https://redirect.github.com/Swatinem/rust-cache/pull/212)\n- Adds an option to do lookup-only of the cache by\n[@&#8203;danlec](https://redirect.github.com/danlec) in\n[#&#8203;217](https://redirect.github.com/Swatinem/rust-cache/pull/217)\n- add runner OS in cache key by\n[@&#8203;rnbguy](https://redirect.github.com/rnbguy) in\n[#&#8203;220](https://redirect.github.com/Swatinem/rust-cache/pull/220)\n- Allow opting out of caching $CARGO\\_HOME/bin. by\n[@&#8203;benjyw](https://redirect.github.com/benjyw) in\n[#&#8203;216](https://redirect.github.com/Swatinem/rust-cache/pull/216)\n\n##### New Contributors\n\n- [@&#8203;guylamar2006](https://redirect.github.com/guylamar2006) made\ntheir first contribution in\n[#&#8203;212](https://redirect.github.com/Swatinem/rust-cache/pull/212)\n- [@&#8203;danlec](https://redirect.github.com/danlec) made their first\ncontribution in\n[#&#8203;217](https://redirect.github.com/Swatinem/rust-cache/pull/217)\n- [@&#8203;rnbguy](https://redirect.github.com/rnbguy) made their first\ncontribution in\n[#&#8203;220](https://redirect.github.com/Swatinem/rust-cache/pull/220)\n- [@&#8203;benjyw](https://redirect.github.com/benjyw) made their first\ncontribution in\n[#&#8203;216](https://redirect.github.com/Swatinem/rust-cache/pull/216)\n\n**Full Changelog**:\n<https://github.com/Swatinem/rust-cache/compare/v2.7.5...v2.7.6>\n\n###\n[`v2.7.5`](https://redirect.github.com/Swatinem/rust-cache/releases/tag/v2.7.5)\n\n[Compare\nSource](https://redirect.github.com/Swatinem/rust-cache/compare/v2.7.3...v2.7.5)\n\n##### What's Changed\n\n- Upgrade checkout action from version 3 to 4 by\n[@&#8203;carsten-wenderdel](https://redirect.github.com/carsten-wenderdel)\nin\n[#&#8203;190](https://redirect.github.com/Swatinem/rust-cache/pull/190)\n- fix: usage of `deprecated` version of `node` by\n[@&#8203;hamirmahal](https://redirect.github.com/hamirmahal) in\n[#&#8203;197](https://redirect.github.com/Swatinem/rust-cache/pull/197)\n- Only run macOsWorkaround() on macOS by\n[@&#8203;heksesang](https://redirect.github.com/heksesang) in\n[#&#8203;206](https://redirect.github.com/Swatinem/rust-cache/pull/206)\n- Support Cargo.lock format cargo-lock v4 by\n[@&#8203;NobodyXu](https://redirect.github.com/NobodyXu) in\n[#&#8203;211](https://redirect.github.com/Swatinem/rust-cache/pull/211)\n\n##### New Contributors\n\n-\n[@&#8203;carsten-wenderdel](https://redirect.github.com/carsten-wenderdel)\nmade their first contribution in\n[#&#8203;190](https://redirect.github.com/Swatinem/rust-cache/pull/190)\n- [@&#8203;hamirmahal](https://redirect.github.com/hamirmahal) made\ntheir first contribution in\n[#&#8203;197](https://redirect.github.com/Swatinem/rust-cache/pull/197)\n- [@&#8203;heksesang](https://redirect.github.com/heksesang) made their\nfirst contribution in\n[#&#8203;206](https://redirect.github.com/Swatinem/rust-cache/pull/206)\n\n**Full Changelog**:\n<https://github.com/Swatinem/rust-cache/compare/v2.7.3...v2.7.5>\n\n###\n[`v2.7.3`](https://redirect.github.com/Swatinem/rust-cache/releases/tag/v2.7.3)\n\n[Compare\nSource](https://redirect.github.com/Swatinem/rust-cache/compare/v2.7.2...v2.7.3)\n\n- Work around upstream problem that causes cache saving to hang for\nminutes.\n\n**Full Changelog**:\n<https://github.com/Swatinem/rust-cache/compare/v2.7.2...v2.7.3>\n\n###\n[`v2.7.2`](https://redirect.github.com/Swatinem/rust-cache/releases/tag/v2.7.2)\n\n[Compare\nSource](https://redirect.github.com/Swatinem/rust-cache/compare/v2.7.1...v2.7.2)\n\n##### What's Changed\n\n- Update action runtime to `node20` by\n[@&#8203;rhysd](https://redirect.github.com/rhysd) in\n[#&#8203;175](https://redirect.github.com/Swatinem/rust-cache/pull/175)\n- Only key by `Cargo.toml` and `Cargo.lock` files of workspace members\nby [@&#8203;max-heller](https://redirect.github.com/max-heller) in\n[#&#8203;180](https://redirect.github.com/Swatinem/rust-cache/pull/180)\n\n##### New Contributors\n\n- [@&#8203;rhysd](https://redirect.github.com/rhysd) made their first\ncontribution in\n[#&#8203;175](https://redirect.github.com/Swatinem/rust-cache/pull/175)\n- [@&#8203;max-heller](https://redirect.github.com/max-heller) made\ntheir first contribution in\n[#&#8203;180](https://redirect.github.com/Swatinem/rust-cache/pull/180)\n\n**Full Changelog**:\n<https://github.com/Swatinem/rust-cache/compare/v2.7.1...v2.7.2>\n\n###\n[`v2.7.1`](https://redirect.github.com/Swatinem/rust-cache/compare/v2.7.0...v2.7.1)\n\n[Compare\nSource](https://redirect.github.com/Swatinem/rust-cache/compare/v2.7.0...v2.7.1)\n\n###\n[`v2.7.0`](https://redirect.github.com/Swatinem/rust-cache/releases/tag/v2.7.0)\n\n[Compare\nSource](https://redirect.github.com/Swatinem/rust-cache/compare/v2.6.2...v2.7.0)\n\n##### What's Changed\n\n- Fix save-if documentation in readme by\n[@&#8203;rukai](https://redirect.github.com/rukai) in\n[#&#8203;166](https://redirect.github.com/Swatinem/rust-cache/pull/166)\n- Support for `trybuild` and similar macro testing tools by\n[@&#8203;neysofu](https://redirect.github.com/neysofu) in\n[#&#8203;168](https://redirect.github.com/Swatinem/rust-cache/pull/168)\n\n##### New Contributors\n\n- [@&#8203;rukai](https://redirect.github.com/rukai) made their first\ncontribution in\n[#&#8203;166](https://redirect.github.com/Swatinem/rust-cache/pull/166)\n- [@&#8203;neysofu](https://redirect.github.com/neysofu) made their\nfirst contribution in\n[#&#8203;168](https://redirect.github.com/Swatinem/rust-cache/pull/168)\n\n**Full Changelog**:\n<https://github.com/Swatinem/rust-cache/compare/v2.6.2...v2.7.0>\n\n###\n[`v2.6.2`](https://redirect.github.com/Swatinem/rust-cache/releases/tag/v2.6.2)\n\n[Compare\nSource](https://redirect.github.com/Swatinem/rust-cache/compare/v2.6.1...v2.6.2)\n\n##### What's Changed\n\n- dep: Use `smol-toml` instead of `toml` by\n[@&#8203;NobodyXu](https://redirect.github.com/NobodyXu) in\n[#&#8203;164](https://redirect.github.com/Swatinem/rust-cache/pull/164)\n\n**Full Changelog**:\n<https://github.com/Swatinem/rust-cache/compare/v2...v2.6.2>\n\n###\n[`v2.6.1`](https://redirect.github.com/Swatinem/rust-cache/releases/tag/v2.6.1)\n\n[Compare\nSource](https://redirect.github.com/Swatinem/rust-cache/compare/v2.6.0...v2.6.1)\n\n- Fix hash contributions of `Cargo.lock`/`Cargo.toml` files.\n\n###\n[`v2.6.0`](https://redirect.github.com/Swatinem/rust-cache/releases/tag/v2.6.0)\n\n[Compare\nSource](https://redirect.github.com/Swatinem/rust-cache/compare/v2.5.1...v2.6.0)\n\n##### What's Changed\n\n- Add \"buildjet\" as a second `cache-provider` backend\n[@&#8203;joroshiba](https://redirect.github.com/joroshiba) in\n[#&#8203;154](https://redirect.github.com/Swatinem/rust-cache/pull/154)\n- Clean up sparse registry index.\n- Do not clean up src of `-sys` crates.\n- Remove `.cargo/credentials.toml` before saving.\n\n##### New Contributors\n\n- [@&#8203;joroshiba](https://redirect.github.com/joroshiba) made their\nfirst contribution in\n[#&#8203;154](https://redirect.github.com/Swatinem/rust-cache/pull/154)\n\n**Full Changelog**:\n<https://github.com/Swatinem/rust-cache/compare/v2.5.1...v2.6.0>\n\n###\n[`v2.5.1`](https://redirect.github.com/Swatinem/rust-cache/releases/tag/v2.5.1)\n\n[Compare\nSource](https://redirect.github.com/Swatinem/rust-cache/compare/v2.5.0...v2.5.1)\n\n- Fix hash contribution of `Cargo.lock`.\n\n###\n[`v2.5.0`](https://redirect.github.com/Swatinem/rust-cache/releases/tag/v2.5.0)\n\n[Compare\nSource](https://redirect.github.com/Swatinem/rust-cache/compare/v2.4.0...v2.5.0)\n\n##### What's Changed\n\n- feat: Rm workspace crates version before caching by\n[@&#8203;NobodyXu](https://redirect.github.com/NobodyXu) in\n[#&#8203;147](https://redirect.github.com/Swatinem/rust-cache/pull/147)\n- feat: Add hash of `.cargo/config.toml` to key by\n[@&#8203;NobodyXu](https://redirect.github.com/NobodyXu) in\n[#&#8203;149](https://redirect.github.com/Swatinem/rust-cache/pull/149)\n\n##### New Contributors\n\n- [@&#8203;NobodyXu](https://redirect.github.com/NobodyXu) made their\nfirst contribution in\n[#&#8203;147](https://redirect.github.com/Swatinem/rust-cache/pull/147)\n\n**Full Changelog**:\n<https://github.com/Swatinem/rust-cache/compare/v2.4.0...v2.5.0>\n\n###\n[`v2.4.0`](https://redirect.github.com/Swatinem/rust-cache/releases/tag/v2.4.0)\n\n[Compare\nSource](https://redirect.github.com/Swatinem/rust-cache/compare/v2.3.0...v2.4.0)\n\n- Fix cache key stability.\n- Use 8 character hash components to reduce the key length, making it\nmore readable.\n\n###\n[`v2.3.0`](https://redirect.github.com/Swatinem/rust-cache/releases/tag/v2.3.0)\n\n[Compare\nSource](https://redirect.github.com/Swatinem/rust-cache/compare/v2.2.1...v2.3.0)\n\n- Add `cache-all-crates` option, which enables caching of crates\ninstalled by workflows.\n- Add installed packages to cache key, so changes to workflows that\ninstall rust tools are detected and cached properly.\n- Fix cache restore failures due to upstream bug.\n- Fix `EISDIR` error due to globed directories.\n- Update runtime `@actions/cache`, `@actions/io` and dev `typescript`\ndependencies.\n- Update `npm run prepare` so it creates distribution files with the\nright line endings.\n\n###\n[`v2.2.1`](https://redirect.github.com/Swatinem/rust-cache/releases/tag/v2.2.1)\n\n[Compare\nSource](https://redirect.github.com/Swatinem/rust-cache/compare/v2.2.0...v2.2.1)\n\n- Update `@actions/cache` dependency to fix usage of `zstd` compression.\n\n###\n[`v2.2.0`](https://redirect.github.com/Swatinem/rust-cache/releases/tag/v2.2.0)\n\n[Compare\nSource](https://redirect.github.com/Swatinem/rust-cache/compare/v2.1.0...v2.2.0)\n\n- Add new `save-if` option to always restore, but only conditionally\nsave the cache.\n\n###\n[`v2.1.0`](https://redirect.github.com/Swatinem/rust-cache/releases/tag/v2.1.0)\n\n[Compare\nSource](https://redirect.github.com/Swatinem/rust-cache/compare/v2.0.2...v2.1.0)\n\n- Only hash `Cargo.{lock,toml}` files in the configured workspace\ndirectories.\n\n###\n[`v2.0.2`](https://redirect.github.com/Swatinem/rust-cache/releases/tag/v2.0.2)\n\n[Compare\nSource](https://redirect.github.com/Swatinem/rust-cache/compare/v2.0.1...v2.0.2)\n\n- Avoid calling cargo metadata on pre-cleanup.\n- Added `prefix-key`, `cache-directories` and `cache-targets` options.\n\n###\n[`v2.0.1`](https://redirect.github.com/Swatinem/rust-cache/releases/tag/v2.0.1)\n\n[Compare\nSource](https://redirect.github.com/Swatinem/rust-cache/compare/v2...v2.0.1)\n\n- Primarily just updating dependencies to fix GitHub deprecation\nnotices.\n\n</details>\n\n<details>\n<summary>actions/create-github-app-token\n(actions/create-github-app-token)</summary>\n\n###\n[`v3.1.1`](https://redirect.github.com/actions/create-github-app-token/releases/tag/v3.1.1)\n\n[Compare\nSource](https://redirect.github.com/actions/create-github-app-token/compare/v3.1.0...v3.1.1)\n\n##### Bug Fixes\n\n- improve error message when app identifier is empty\n([#&#8203;362](https://redirect.github.com/actions/create-github-app-token/issues/362))\n([07e2b76](https://redirect.github.com/actions/create-github-app-token/commit/07e2b760664f080c40eec4eacf7477256582db36)),\ncloses\n[#&#8203;249](https://redirect.github.com/actions/create-github-app-token/issues/249)\n\n###\n[`v3.1.0`](https://redirect.github.com/actions/create-github-app-token/releases/tag/v3.1.0)\n\n[Compare\nSource](https://redirect.github.com/actions/create-github-app-token/compare/v3...v3.1.0)\n\n##### Bug Fixes\n\n- **deps:** bump p-retry from 7.1.1 to 8.0.0\n([#&#8203;357](https://redirect.github.com/actions/create-github-app-token/issues/357))\n([3bbe07d](https://redirect.github.com/actions/create-github-app-token/commit/3bbe07d928e2d6c30bf3e37c6b89edbc4045facf))\n\n##### Features\n\n- add `client-id` input and deprecate `app-id`\n([#&#8203;353](https://redirect.github.com/actions/create-github-app-token/issues/353))\n([e6bd4e6](https://redirect.github.com/actions/create-github-app-token/commit/e6bd4e6970172bed9fe138b2eaf4cbffa4cca8f9))\n- update permission inputs\n([#&#8203;358](https://redirect.github.com/actions/create-github-app-token/issues/358))\n([076e948](https://redirect.github.com/actions/create-github-app-token/commit/076e9480ca6e9633bff412d05eff0fc2f1e7d2be))\n\n</details>\n\n<details>\n<summary>actions/setup-node (actions/setup-node)</summary>\n\n###\n[`v6.4.0`](https://redirect.github.com/actions/setup-node/compare/v6.3.0...v6.4.0)\n\n[Compare\nSource](https://redirect.github.com/actions/setup-node/compare/v6.3.0...v6.4.0)\n\n</details>\n\n<details>\n<summary>actions/upload-artifact (actions/upload-artifact)</summary>\n\n###\n[`v7.0.1`](https://redirect.github.com/actions/upload-artifact/releases/tag/v7.0.1)\n\n[Compare\nSource](https://redirect.github.com/actions/upload-artifact/compare/v7...v7.0.1)\n\n##### What's Changed\n\n- Update the readme with direct upload details by\n[@&#8203;danwkennedy](https://redirect.github.com/danwkennedy) in\n[#&#8203;795](https://redirect.github.com/actions/upload-artifact/pull/795)\n- Readme: bump all the example versions to v7 by\n[@&#8203;danwkennedy](https://redirect.github.com/danwkennedy) in\n[#&#8203;796](https://redirect.github.com/actions/upload-artifact/pull/796)\n- Include changes in typespec/ts-http-runtime 0.3.5 by\n[@&#8203;yacaovsnc](https://redirect.github.com/yacaovsnc) in\n[#&#8203;797](https://redirect.github.com/actions/upload-artifact/pull/797)\n\n**Full Changelog**:\n<https://github.com/actions/upload-artifact/compare/v7...v7.0.1>\n\n</details>\n\n<details>\n<summary>github/codeql-action (github/codeql-action)</summary>\n\n###\n[`v4.35.3`](https://redirect.github.com/github/codeql-action/releases/tag/v4.35.3)\n\n[Compare\nSource](https://redirect.github.com/github/codeql-action/compare/v4.35.2...v4.35.3)\n\n- *Upcoming breaking change*: Add a deprecation warning for customers\nusing CodeQL version 2.19.3 and earlier. These versions of CodeQL were\ndiscontinued on 9 April 2026 alongside GitHub Enterprise Server 3.15,\nand will be unsupported by the next minor release of the CodeQL Action.\n[#&#8203;3837](https://redirect.github.com/github/codeql-action/pull/3837)\n- Configurations for private registries that use Cloudsmith or GCP OIDC\nare now accepted.\n[#&#8203;3850](https://redirect.github.com/github/codeql-action/pull/3850)\n- Best-effort connection tests for private registries now use `GET`\nrequests instead of `HEAD` for better compatibility with various\nregistry implementations. For NuGet feeds, the test is now always\nperformed against the service index.\n[#&#8203;3853](https://redirect.github.com/github/codeql-action/pull/3853)\n- Fixed a bug where two diagnostics produced within the same millisecond\ncould overwrite each other on disk, causing one of them to be lost.\n[#&#8203;3852](https://redirect.github.com/github/codeql-action/pull/3852)\n- Update default CodeQL bundle version to\n[2.25.3](https://redirect.github.com/github/codeql-action/releases/tag/codeql-bundle-v2.25.3).\n[#&#8203;3865](https://redirect.github.com/github/codeql-action/pull/3865)\n\n###\n[`v4.35.2`](https://redirect.github.com/github/codeql-action/releases/tag/v4.35.2)\n\n[Compare\nSource](https://redirect.github.com/github/codeql-action/compare/v4.35.1...v4.35.2)\n\n- The undocumented TRAP cache cleanup feature that could be enabled\nusing the `CODEQL_ACTION_CLEANUP_TRAP_CACHES` environment variable is\ndeprecated and will be removed in May 2026. If you are affected by this,\nwe recommend disabling TRAP caching by passing the `trap-caching: false`\ninput to the `init` Action.\n[#&#8203;3795](https://redirect.github.com/github/codeql-action/pull/3795)\n- The Git version 2.36.0 requirement for improved incremental analysis\nnow only applies to repositories that contain submodules.\n[#&#8203;3789](https://redirect.github.com/github/codeql-action/pull/3789)\n- Python analysis on GHES no longer extracts the standard library,\nrelying instead on models of the standard library. This should result in\nsignificantly faster extraction and analysis times, while the effect on\nalerts should be minimal.\n[#&#8203;3794](https://redirect.github.com/github/codeql-action/pull/3794)\n- Fixed a bug in the validation of OIDC configurations for private\nregistries that was added in CodeQL Action 4.33.0 / 3.33.0.\n[#&#8203;3807](https://redirect.github.com/github/codeql-action/pull/3807)\n- Update default CodeQL bundle version to\n[2.25.2](https://redirect.github.com/github/codeql-action/releases/tag/codeql-bundle-v2.25.2).\n[#&#8203;3823](https://redirect.github.com/github/codeql-action/pull/3823)\n\n</details>\n\n<details>\n<summary>actions/go-versions (go)</summary>\n\n###\n[`v1.26.2`](https://redirect.github.com/actions/go-versions/releases/tag/1.26.2-24114135105):\n1.26.2\n\n[Compare\nSource](https://redirect.github.com/actions/go-versions/compare/1.26.1-22746851271...1.26.2-24114135105)\n\nGo 1.26.2\n\n</details>\n\n<details>\n<summary>step-security/harden-runner\n(step-security/harden-runner)</summary>\n\n###\n[`v2.19.0`](https://redirect.github.com/step-security/harden-runner/releases/tag/v2.19.0)\n\n[Compare\nSource](https://redirect.github.com/step-security/harden-runner/compare/v2.18.0...v2.19.0)\n\n##### What's Changed\n\n##### New Runner Support\n\nHarden-Runner now supports Depot, Blacksmith, Namespace, and WarpBuild\nrunners with the same egress monitoring, runtime monitoring, and policy\nenforcement available on GitHub-hosted runners.\n\n##### Automated Incident Response for Supply Chain Attacks\n\n- Global block list: Outbound connections to known malicious domains and\nIPs are now blocked even in audit mode.\n- System-defined detection rules: Harden-Runner will trigger lockdown\nmode when a high risk event is detected during an active supply chain\nattack (for example, a process reading the memory of the runner worker\nprocess, a common technique for stealing GitHub Actions secrets).\n\n##### Bug Fixes\n\nWindows and macOS: stability and reliability fixes\n\n**Full Changelog**:\n<https://github.com/step-security/harden-runner/compare/v2.18.0...v2.19.0>\n\n###\n[`v2.18.0`](https://redirect.github.com/step-security/harden-runner/releases/tag/v2.18.0)\n\n[Compare\nSource](https://redirect.github.com/step-security/harden-runner/compare/v2.17.0...v2.18.0)\n\n##### What's Changed\n\nGlobal Block List: During supply chain incidents like the recent axios\nand trivy compromises, StepSecurity will add known malicious domains and\nIP addresses (IOCs) to a global block list. These will be automatically\nblocked, even in audit mode, providing immediate protection without\nrequiring any workflow changes.\n\nDeploy on Self-Hosted VM: Added `deploy-on-self-hosted-vm` input that\nallows the Harden Runner agent to be installed directly on ephemeral\nself-hosted Linux runner VMs at workflow runtime. This is intended as an\nalternative when baking the agent into the VM image is not possible.\n\n**Full Changelog**:\n<https://github.com/step-security/harden-runner/compare/v2.17.0...v2.18.0>\n\n###\n[`v2.17.0`](https://redirect.github.com/step-security/harden-runner/releases/tag/v2.17.0)\n\n[Compare\nSource](https://redirect.github.com/step-security/harden-runner/compare/v2.16.1...v2.17.0)\n\n##### What's Changed\n\n##### Policy Store Support\n\nAdded `use-policy-store` and `api-key` inputs to fetch security policies\ndirectly from the [StepSecurity Policy\nStore](https://docs.stepsecurity.io/harden-runner/policy-store).\nPolicies can be defined and attached at the workflow, repo, org, or\ncluster (ARC) level, with the most granular policy taking precedence.\nThis is the preferred method over the existing `policy` input which\nrequires `id-token: write` permission. If no policy is found in the\nstore, the action defaults to audit mode.\n\n**Full Changelog**:\n<https://github.com/step-security/harden-runner/compare/v2.16.1...v2.17.0>\n\n</details>\n\n<details>\n<summary>taiki-e/install-action (taiki-e/install-action)</summary>\n\n###\n[`v2.75.28`](https://redirect.github.com/taiki-e/install-action/compare/v2.75.27...v2.75.28)\n\n[Compare\nSource](https://redirect.github.com/taiki-e/install-action/compare/v2.75.27...v2.75.28)\n\n###\n[`v2.75.27`](https://redirect.github.com/taiki-e/install-action/releases/tag/v2.75.27):\n2.75.27\n\n[Compare\nSource](https://redirect.github.com/taiki-e/install-action/compare/v2.75.26...v2.75.27)\n\n- Update `cargo-udeps@latest` to 0.1.61.\n\n- Update `wasm-tools@latest` to 1.248.0.\n\n- Update `cargo-deb@latest` to 3.6.4.\n\n###\n[`v2.75.26`](https://redirect.github.com/taiki-e/install-action/releases/tag/v2.75.26):\n2.75.26\n\n[Compare\nSource](https://redirect.github.com/taiki-e/install-action/compare/v2.75.25...v2.75.26)\n\n- Update `wasm-bindgen@latest` to 0.2.120.\n\n- Update `mise@latest` to 2026.4.25.\n\n- Update `martin@latest` to 1.8.0.\n\n- Update `vacuum@latest` to 0.26.4.\n\n###\n[`v2.75.25`](https://redirect.github.com/taiki-e/install-action/releases/tag/v2.75.25):\n2.75.25\n\n[Compare\nSource](https://redirect.github.com/taiki-e/install-action/compare/v2.75.24...v2.75.25)\n\n- Update `uv@latest` to 0.11.8.\n\n- Update `typos@latest` to 1.45.2.\n\n- Update `tombi@latest` to 0.9.25.\n\n- Update `mise@latest` to 2026.4.24.\n\n###\n[`v2.75.24`](https://redirect.github.com/taiki-e/install-action/releases/tag/v2.75.24):\n2.75.24\n\n[Compare\nSource](https://redirect.github.com/taiki-e/install-action/compare/v2.75.23...v2.75.24)\n\n- Update `prek@latest` to 0.3.11.\n\n- Update `mise@latest` to 2026.4.23.\n\n- Update `vacuum@latest` to 0.26.3.\n\n###\n[`v2.75.23`](https://redirect.github.com/taiki-e/install-action/releases/tag/v2.75.23):\n2.75.23\n\n[Compare\nSource](https://redirect.github.com/taiki-e/install-action/compare/v2.75.22...v2.75.23)\n\n- Update `vacuum@latest` to 0.26.2.\n\n- Update `tombi@latest` to 0.9.24.\n\n- Update `mise@latest` to 2026.4.22.\n\n- Update `martin@latest` to 1.7.0.\n\n- Update `git-cliff@latest` to 2.13.1.\n\n- Update `cargo-tarpaulin@latest` to 0.35.4.\n\n- Update `cargo-sort@latest` to 2.1.4.\n\n###\n[`v2.75.22`](https://redirect.github.com/taiki-e/install-action/releases/tag/v2.75.22):\n2.75.22\n\n[Compare\nSource](https://redirect.github.com/taiki-e/install-action/compare/v2.75.21...v2.75.22)\n\n- Update `tombi@latest` to 0.9.22.\n\n- Update `biome@latest` to 2.4.13.\n\n###\n[`v2.75.21`](https://redirect.github.com/taiki-e/install-action/releases/tag/v2.75.21):\n2.75.21\n\n[Compare\nSource](https://redirect.github.com/taiki-e/install-action/compare/v2.75.20...v2.75.21)\n\n- Update `mise@latest` to 2026.4.19.\n\n- Update `tombi@latest` to 0.9.21.\n\n- Update `syft@latest` to 1.43.0.\n\n###\n[`v2.75.20`](https://redirect.github.com/taiki-e/install-action/releases/tag/v2.75.20):\n2.75.20\n\n[Compare\nSource](https://redirect.github.com/taiki-e/install-action/compare/v2.75.19...v2.75.20)\n\n- Update `prek@latest` to 0.3.10.\n\n- Update `cargo-xwin@latest` to 0.22.0.\n\n###\n[`v2.75.19`](https://redirect.github.com/taiki-e/install-action/releases/tag/v2.75.19):\n2.75.19\n\n[Compare\nSource](https://redirect.github.com/taiki-e/install-action/compare/v2.75.18...v2.75.19)\n\n- Update `wasmtime@latest` to 44.0.0.\n\n- Update `tombi@latest` to 0.9.20.\n\n- Update `martin@latest` to 1.6.0.\n\n- Update `just@latest` to 1.50.0.\n\n- Update `mise@latest` to 2026.4.18.\n\n- Update `rclone@latest` to 1.73.5.\n\n###\n[`v2.75.18`](https://redirect.github.com/taiki-e/install-action/releases/tag/v2.75.18):\n2.75.18\n\n[Compare\nSource](https://redirect.github.com/taiki-e/install-action/compare/v2.75.17...v2.75.18)\n\n- Update `vacuum@latest` to 0.26.1.\n\n- Update `wasm-tools@latest` to 1.247.0.\n\n- Update `mise@latest` to 2026.4.16.\n\n- Update `espup@latest` to 0.17.1.\n\n- Update `trivy@latest` to 0.70.0.\n\n###\n[`v2.75.17`](https://redirect.github.com/taiki-e/install-action/blob/HEAD/CHANGELOG.md#100---2021-12-30)\n\n[Compare\nSource](https://redirect.github.com/taiki-e/install-action/compare/v2.75.16...v2.75.17)\n\nInitial release\n\n[Unreleased]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.75.21...HEAD\n\n[2.75.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.75.20...v2.75.21\n\n[2.75.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.75.19...v2.75.20\n\n[2.75.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.75.18...v2.75.19\n\n[2.75.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.75.17...v2.75.18\n\n[2.75.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.75.16...v2.75.17\n\n[2.75.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.75.15...v2.75.16\n\n[2.75.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.75.14...v2.75.15\n\n[2.75.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.75.13...v2.75.14\n\n[2.75.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.75.12...v2.75.13\n\n[2.75.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.75.11...v2.75.12\n\n[2.75.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.75.10...v2.75.11\n\n[2.75.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.75.9...v2.75.10\n\n[2.75.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.75.8...v2.75.9\n\n[2.75.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.75.7...v2.75.8\n\n[2.75.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.75.6...v2.75.7\n\n[2.75.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.75.5...v2.75.6\n\n[2.75.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.75.4...v2.75.5\n\n[2.75.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.75.3...v2.75.4\n\n[2.75.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.75.2...v2.75.3\n\n[2.75.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.75.1...v2.75.2\n\n[2.75.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.75.0...v2.75.1\n\n[2.75.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.74.1...v2.75.0\n\n[2.74.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.74.0...v2.74.1\n\n[2.74.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.73.0...v2.74.0\n\n[2.73.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.72.0...v2.73.0\n\n[2.72.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.71.3...v2.72.0\n\n[2.71.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.71.2...v2.71.3\n\n[2.71.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.71.1...v2.71.2\n\n[2.71.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.71.0...v2.71.1\n\n[2.71.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.70.4...v2.71.0\n\n[2.70.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.70.3...v2.70.4\n\n[2.70.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.70.2...v2.70.3\n\n[2.70.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.70.1...v2.70.2\n\n[2.70.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.70.0...v2.70.1\n\n[2.70.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.69.14...v2.70.0\n\n[2.69.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.69.13...v2.69.14\n\n[2.69.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.69.12...v2.69.13\n\n[2.69.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.69.11...v2.69.12\n\n[2.69.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.69.10...v2.69.11\n\n[2.69.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.69.9...v2.69.10\n\n[2.69.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.69.8...v2.69.9\n\n[2.69.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.69.7...v2.69.8\n\n[2.69.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.69.6...v2.69.7\n\n[2.69.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.69.5...v2.69.6\n\n[2.69.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.69.4...v2.69.5\n\n[2.69.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.69.3...v2.69.4\n\n[2.69.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.69.2...v2.69.3\n\n[2.69.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.69.1...v2.69.2\n\n[2.69.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.69.0...v2.69.1\n\n[2.69.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.36...v2.69.0\n\n[2.68.36]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.35...v2.68.36\n\n[2.68.35]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.34...v2.68.35\n\n[2.68.34]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.33...v2.68.34\n\n[2.68.33]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.32...v2.68.33\n\n[2.68.32]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.31...v2.68.32\n\n[2.68.31]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.30...v2.68.31\n\n[2.68.30]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.29...v2.68.30\n\n[2.68.29]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.28...v2.68.29\n\n[2.68.28]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.27...v2.68.28\n\n[2.68.27]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.26...v2.68.27\n\n[2.68.26]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.25...v2.68.26\n\n[2.68.25]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.24...v2.68.25\n\n[2.68.24]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.23...v2.68.24\n\n[2.68.23]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.22...v2.68.23\n\n[2.68.22]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.21...v2.68.22\n\n[2.68.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.20...v2.68.21\n\n[2.68.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.19...v2.68.20\n\n[2.68.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.18...v2.68.19\n\n[2.68.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.17...v2.68.18\n\n[2.68.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.16...v2.68.17\n\n[2.68.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.15...v2.68.16\n\n[2.68.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.14...v2.68.15\n\n[2.68.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.13...v2.68.14\n\n[2.68.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.12...v2.68.13\n\n[2.68.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.11...v2.68.12\n\n[2.68.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.10...v2.68.11\n\n[2.68.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.9...v2.68.10\n\n[2.68.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.8...v2.68.9\n\n[2.68.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.7...v2.68.8\n\n[2.68.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.6...v2.68.7\n\n[2.68.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.5...v2.68.6\n\n[2.68.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.4...v2.68.5\n\n[2.68.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.3...v2.68.4\n\n[2.68.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.2...v2.68.3\n\n[2.68.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.1...v2.68.2\n\n[2.68.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.0...v2.68.1\n\n[2.68.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.30...v2.68.0\n\n[2.67.30]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.29...v2.67.30\n\n[2.67.29]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.28...v2.67.29\n\n[2.67.28]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.27...v2.67.28\n\n[2.67.27]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.26...v2.67.27\n\n[2.67.26]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.25...v2.67.26\n\n[2.67.25]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.24...v2.67.25\n\n[2.67.24]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.23...v2.67.24\n\n[2.67.23]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.22...v2.67.23\n\n[2.67.22]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.21...v2.67.22\n\n[2.67.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.20...v2.67.21\n\n[2.67.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.19...v2.67.20\n\n[2.67.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.18...v2.67.19\n\n[2.67.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.17...v2.67.18\n\n[2.67.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.16...v2.67.17\n\n[2.67.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.15...v2.67.16\n\n[2.67.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.14...v2.67.15\n\n[2.67.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.13...v2.67.14\n\n[2.67.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.12...v2.67.13\n\n[2.67.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.11...v2.67.12\n\n[2.67.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.10...v2.67.11\n\n[2.67.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.9...v2.67.10\n\n[2.67.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.8...v2.67.9\n\n[2.67.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.7...v2.67.8\n\n[2.67.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.6...v2.67.7\n\n[2.67.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.5...v2.67.6\n\n[2.67.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.4...v2.67.5\n\n[2.67.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.3...v2.67.4\n\n[2.67.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.2...v2.67.3\n\n[2.67.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.1...v2.67.2\n\n[2.67.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.0...v2.67.1\n\n[2.67.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.66.7...v2.67.0\n\n[2.66.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.66.6...v2.66.7\n\n[2.66.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.66.5...v2.66.6\n\n[2.66.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.66.4...v2.66.5\n\n[2.66.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.66.3...v2.66.4\n\n[2.66.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.66.2...v2.66.3\n\n[2.66.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.66.1...v2.66.2\n\n[2.66.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.66.0...v2.66.1\n\n[2.66.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.16...v2.66.0\n\n[2.65.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.15...v2.65.16\n\n[2.65.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.14...v2.65.15\n\n[2.65.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.13...v2.65.14\n\n[2.65.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.12...v2.65.13\n\n[2.65.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.11...v2.65.12\n\n[2.65.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.10...v2.65.11\n\n[2.65.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.9...v2.65.10\n\n[2.65.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.8...v2.65.9\n\n[2.65.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.7...v2.65.8\n\n[2.65.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.6...v2.65.7\n\n[2.65.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.5...v2.65.6\n\n[2.65.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.4...v2.65.5\n\n[2.65.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.3...v2.65.4\n\n[2.65.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.2...v2.65.3\n\n[2.65.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.1...v2.65.2\n\n[2.65.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.0...v2.65.1\n\n[2.65.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.64.2...v2.65.0\n\n[2.64.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.64.1...v2.64.2\n\n[2.64.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.64.0...v2.64.1\n\n[2.64.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.63.3...v2.64.0\n\n[2.63.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.63.2...v2.63.3\n\n[2.63.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.63.1...v2.63.2\n\n[2.63.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.63.0...v2.63.1\n\n[2.63.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.67...v2.63.0\n\n[2.62.67]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.66...v2.62.67\n\n[2.62.66]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.65...v2.62.66\n\n[2.62.65]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.64...v2.62.65\n\n[2.62.64]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.63...v2.62.64\n\n[2.62.63]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.62...v2.62.63\n\n[2.62.62]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.61...v2.62.62\n\n[2.62.61]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.60...v2.62.61\n\n[2.62.60]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.59...v2.62.60\n\n[2.62.59]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.58...v2.62.59\n\n[2.62.58]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.57...v2.62.58\n\n[2.62.57]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.56...v2.62.57\n\n[2.62.56]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.55...v2.62.56\n\n[2.62.55]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.54...v2.62.55\n\n[2.62.54]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.53...v2.62.54\n\n[2.62.53]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.52...v2.62.53\n\n[2.62.52]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.51...v2.62.52\n\n[2.62.51]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.50...v2.62.51\n\n[2.62.50]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.49...v2.62.50\n\n[2.62.49]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.48...v2.62.49\n\n[2.62.48]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.47...v2.62.48\n\n[2.62.47]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.46...v2.62.47\n\n[2.62.46]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.45...v2.62.46\n\n[2.62.45]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.44...v2.62.45\n\n[2.62.44]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.43...v2.62.44\n\n[2.62.43]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.42...v2.62.43\n\n[2.62.42]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.41...v2.62.42\n\n[2.62.41]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.40...v2.62.41\n\n[2.62.40]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.39...v2.62.40\n\n[2.62.39]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.38...v2.62.39\n\n[2.62.38]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.37...v2.62.38\n\n[2.62.37]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.36...v2.62.37\n\n[2.62.36]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.35...v2.62.36\n\n[2.62.35]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.34...v2.62.35\n\n[2.62.34]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.33...v2.62.34\n\n[2.62.33]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.32...v2.62.33\n\n[2.62.32]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.31...v2.62.32\n\n[2.62.31]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.30...v2.62.31\n\n[2.62.30]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.29...v2.62.30\n\n[2.62.29]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.28...v2.62.29\n\n[2.62.28]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.27...v2.62.28\n\n[2.62.27]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.26...v2.62.27\n\n[2.62.26]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.25...v2.62.26\n\n[2.62.25]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.24...v2.62.25\n\n[2.62.24]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.23...v2.62.24\n\n[2.62.23]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.22...v2.62.23\n\n[2.62.22]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.21...v2.62.22\n\n[2.62.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.20...v2.62.21\n\n[2.62.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.19...v2.62.20\n\n[2.62.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.18...v2.62.19\n\n[2.62.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.17...v2.62.18\n\n[2.62.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.16...v2.62.17\n\n[2.62.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.15...v2.62.16\n\n[2.62.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.14...v2.62.15\n\n[2.62.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.13...v2.62.14\n\n[2.62.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.12...v2.62\n\n> ✂ **Note**\n> \n> PR body was truncated to here.\n\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: (UTC)\n\n- Branch creation\n  - \"before 8am on the first day of the month\"\n- Automerge\n  - At any time (no schedule defined)\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n👻 **Immortal**: This PR will be recreated if closed unmerged. Get\n[config\nhelp](https://redirect.github.com/renovatebot/renovate/discussions) if\nthat's undesired.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My4xNTkuMiIsInVwZGF0ZWRJblZlciI6IjQzLjE1OS4yIiwidGFyZ2V0QnJhbmNoIjoibWFpbiIsImxhYmVscyI6WyJkZXBlbmRlbmNpZXMiXX0=-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2026-05-02T11:20:16Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/9aa767ee7b26712bbab69e4ecab5db2b22f80f32"
        },
        "date": 1777751365427,
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
            "value": 100.15711689855887,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.34843205574913,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.776302083333334,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.5078125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 0,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 165897.4163254844,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00214,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 904985.3986455925,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 47842899.91881924,
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
            "value": 100.24312574383151,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.50697623685237,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.00703125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.25,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 0,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 114671.04002675108,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002072,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 799958.8611978483,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 33192465.01183511,
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
            "value": 93.01540722873281,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.35850704661607,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 35.07578125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 35.953125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 0,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 111821.07508026468,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002106,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2136888.3770502834,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 32397889.23093702,
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
            "value": 95.29014851866059,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 95.66460370341674,
            "unit": "%",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 30.655729166666667,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 31.12890625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 0,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 250736.6106649905,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003204,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2800084.7760421196,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 73010069.60783084,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Network Utilization"
          }
        ]
      }
    ]
  }
}