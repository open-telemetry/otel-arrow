window.BENCHMARK_DATA = {
  "lastUpdate": 1774490515404,
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
      }
    ]
  }
}