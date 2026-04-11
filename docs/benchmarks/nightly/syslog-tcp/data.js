window.BENCHMARK_DATA = {
  "lastUpdate": 1775873932061,
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
      }
    ]
  }
}