window.BENCHMARK_DATA = {
  "lastUpdate": 1774200103601,
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
      }
    ]
  }
}