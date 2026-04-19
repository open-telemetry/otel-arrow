window.BENCHMARK_DATA = {
  "lastUpdate": 1776564727555,
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
        "date": 1774114733535,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_signals_percentage",
            "value": -1.5714285373687744,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Dropped Metrics %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 17.076646839774202,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 17.978334361505244,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 31.346484375,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 33.5,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "metrics_produced_rate",
            "value": 104996.08714581902,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Metric Throughput"
          },
          {
            "name": "metrics_received_rate",
            "value": 106646.02565811046,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Metric Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002236,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 217436.10947422223,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 921590.3791077056,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.5873017311096191,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 14.293084691238814,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 15.732918047953126,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 31.199088541666665,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 33.16015625,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104995.72142435196,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106662.32017711944,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002445,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 133155.96452499423,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 540982.4736732196,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": -0.4285714030265808,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Dropped Spans %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 30.86811838004583,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 32.22668521094112,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 50.061197916666664,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 52.74609375,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "spans_produced_rate",
            "value": 104995.45894640057,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Span Throughput"
          },
          {
            "name": "spans_received_rate",
            "value": 105445.43948474228,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Span Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002595,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 669126.821582459,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3629329.009024402,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": -1115,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Dropped Metrics %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 16.485650709554704,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 18.827773304428614,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 15.414192708333333,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 16.34765625,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "metrics_produced_rate",
            "value": 1666.6005026267126,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Metric Throughput"
          },
          {
            "name": "metrics_received_rate",
            "value": 20249.19610691456,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Metric Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002382,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 820370.8104016674,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 791638.9327254213,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -112.75862121582031,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 48.679955191354935,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 49.680600479764756,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 14.791536458333333,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 15.5546875,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 48331.421020108304,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 102829.26472209248,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002374,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2384535.737724454,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2359265.603984686,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": -925,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Dropped Spans %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 21.24105059525706,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 22.389972987574282,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 24.191536458333335,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 29.390625,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "spans_produced_rate",
            "value": 1666.6061133112162,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Span Throughput"
          },
          {
            "name": "spans_received_rate",
            "value": 17082.712661439968,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Span Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00218,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2040267.2110330497,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2038450.4044580555,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1230,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 11.658452162476987,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 12.168423020014018,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.384895833333335,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.41796875,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1666.6044189916174,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 22165.83877258851,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002241,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 492480.9071789664,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 487320.7739329305,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": -1.5873017311096191,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Dropped Metrics %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 16.638755149238175,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 17.871816854882077,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.621744791666668,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.65234375,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "metrics_produced_rate",
            "value": 104995.9086594259,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Metric Throughput"
          },
          {
            "name": "metrics_received_rate",
            "value": 106662.51038417869,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Metric Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002338,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 965394.9919276127,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 944558.2311920275,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": -2516,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Dropped Spans %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 47.534120165350394,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 54.26349058072552,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.253776041666665,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.22265625,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "spans_produced_rate",
            "value": 1666.6044467673207,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Span Throughput"
          },
          {
            "name": "spans_received_rate",
            "value": 43598.372327433106,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Span Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00224,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 5517724.8910489315,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5493716.7063547075,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": -4111,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Dropped Metrics %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 47.584327471649075,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 48.69981586073501,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 14.230338541666667,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 14.96484375,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "metrics_produced_rate",
            "value": 1666.6021136114662,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Metric Throughput"
          },
          {
            "name": "metrics_received_rate",
            "value": 70180.61500417884,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Metric Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002324,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2959999.297631531,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2930557.958639193,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": -0.460317462682724,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Dropped Spans %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 27.30224562698088,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 28.61711479435236,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 24.241015625,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 25.34375,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "spans_produced_rate",
            "value": 104995.64968024826,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Span Throughput"
          },
          {
            "name": "spans_received_rate",
            "value": 105478.96298830019,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Span Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002486,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3653152.7867686595,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3632304.5285975663,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.5873017311096191,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 16.530589664445987,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 18.01955009596929,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.456119791666666,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 21.78125,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104996.12039335146,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106662.72547896022,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002217,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 566729.3563627049,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 546641.1308029385,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Network Utilization"
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
        "date": 1774144775207,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -52.07316970825195,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 48.60001571313353,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 50.10879727743832,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 14.308984375,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 14.859375,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 68330.62971808415,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 103912.55519323285,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002374,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2451968.959323192,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2428753.3436761405,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": -1.5714285373687744,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Dropped Metrics %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 17.06750376569344,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 18.173129156999227,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 30.908984375,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 32.82421875,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "metrics_produced_rate",
            "value": 104995.68292750363,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Metric Throughput"
          },
          {
            "name": "metrics_received_rate",
            "value": 106645.61508779296,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Metric Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002467,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 215659.6101528601,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 900797.617873752,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.5873017311096191,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 16.4789999427543,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 17.35721497491316,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.47734375,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 21.85546875,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104996.09589516763,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106662.70059191632,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002231,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 562011.5197406871,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 541556.2871885706,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.5714285373687744,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 14.252956023787961,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 15.814907749077491,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 30.861979166666668,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 33.359375,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104995.1632228142,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106645.08721631557,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002764,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 133484.7954173117,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 543905.3406484734,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": -2750,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Dropped Spans %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 52.07366690800851,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 53.92019789734075,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.554296875,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.94921875,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "spans_produced_rate",
            "value": 1666.602613572885,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Span Throughput"
          },
          {
            "name": "spans_received_rate",
            "value": 47498.17448682722,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Span Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002306,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 6092153.37935772,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 6076831.913027227,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Dropped Spans %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 27.573002498772215,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 29.172010201715743,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 24.653515625,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 25.50390625,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "spans_produced_rate",
            "value": 104995.49569323476,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Span Throughput"
          },
          {
            "name": "spans_received_rate",
            "value": 104995.49569323476,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Span Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002574,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3643161.0085611404,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3637045.1726515125,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1235,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 11.565614291394317,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 12.281062330623307,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 15.388541666666667,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 15.99609375,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1666.6050856087534,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 22249.177892876858,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002217,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 489300.9215031929,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 481201.1488457898,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": -0.4285714030265808,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Dropped Spans %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 31.007971247952298,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 33.12298613800046,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 49.379947916666666,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 52.984375,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "spans_produced_rate",
            "value": 104996.02240068471,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Span Throughput"
          },
          {
            "name": "spans_received_rate",
            "value": 105446.00535383051,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Span Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002273,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 668123.0575175914,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3619874.2397100325,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": -1110,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Dropped Metrics %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 16.575989715005626,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 18.957714639622203,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 15.354817708333334,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 16.5078125,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "metrics_produced_rate",
            "value": 1666.6010025871647,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Metric Throughput"
          },
          {
            "name": "metrics_received_rate",
            "value": 20165.87213130469,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Metric Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002364,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 816483.0940020446,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 788107.9746830618,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": -1.5873017311096191,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Dropped Metrics %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 16.667604553159148,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 18.310984696243622,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.529817708333333,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.38671875,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "metrics_produced_rate",
            "value": 104996.1291427056,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Metric Throughput"
          },
          {
            "name": "metrics_received_rate",
            "value": 106662.734367193,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Metric Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002212,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 943936.6267795481,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 922811.1183347126,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": -925,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Dropped Spans %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 21.287451386954945,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 22.14898366746652,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 24.477864583333332,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 26.40234375,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "spans_produced_rate",
            "value": 1666.6014469967076,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Span Throughput"
          },
          {
            "name": "spans_received_rate",
            "value": 17082.664831716254,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Span Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002348,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2032118.4816722523,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2028441.7788535855,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": -4081.000244140625,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Dropped Metrics %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 47.62120705703411,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 48.27650796092608,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 13.784244791666667,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 14.21875,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "metrics_produced_rate",
            "value": 1666.6033357399085,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Metric Throughput"
          },
          {
            "name": "metrics_received_rate",
            "value": 69680.68546728558,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Metric Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00228,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2956717.35902658,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2930325.4010586087,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Network Utilization"
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
        "date": 1774200111513,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_signals_percentage",
            "value": -925,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Dropped Spans %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 21.420698557093353,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 22.395449064770443,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 23.417838541666665,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 26.01953125,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "spans_produced_rate",
            "value": 1666.6037245993343,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Span Throughput"
          },
          {
            "name": "spans_received_rate",
            "value": 17082.688177143176,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Span Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002266,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2031290.2862126206,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2030206.5829892445,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": -2676,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Dropped Spans %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 50.63709089323092,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 54.06055878028017,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.398177083333334,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.6953125,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "spans_produced_rate",
            "value": 1666.601085913936,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Span Throughput"
          },
          {
            "name": "spans_received_rate",
            "value": 46264.84614497086,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Span Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002361,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 5875107.754943101,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5863737.058755014,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": -0.4444444477558136,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Dropped Spans %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 27.98503299695161,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 29.278056178905825,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 24.137109375,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 26.86328125,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "spans_produced_rate",
            "value": 104995.9401569806,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Span Throughput"
          },
          {
            "name": "spans_received_rate",
            "value": 105462.58877990051,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Span Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00232,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3664645.893933737,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3658649.586206017,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Dropped Spans %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 30.846776411816535,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 32.17777503281093,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 49.18528645833333,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 52.12109375,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "spans_produced_rate",
            "value": 104995.80891729405,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Span Throughput"
          },
          {
            "name": "spans_received_rate",
            "value": 104995.80891729405,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Span Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002395,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 669516.4643647447,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3605192.7063896237,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.5873017311096191,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 16.37417102404172,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 17.736606743306844,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.811328125,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 21.15625,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104995.98040388354,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106662.58326743725,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002297,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 564418.6468072032,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 544371.1284460879,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1250,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 11.5296268600283,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 12.312319603592444,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 15.667838541666667,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.11328125,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1666.60216916272,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 22499.129283696722,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002322,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 493074.0461790447,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 483195.01527914574,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": -4781,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Dropped Metrics %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 50.21443418744318,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 53.295661997988084,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 14.275651041666666,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 15.05859375,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "metrics_produced_rate",
            "value": 1666.6031135346038,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Metric Throughput"
          },
          {
            "name": "metrics_received_rate",
            "value": 81346.89797162401,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Metric Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002288,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3429648.3315145182,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3407891.346935555,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 14.197458547605823,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 15.666149216881415,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 30.676822916666666,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 33.04296875,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104995.71267506576,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106045.66980181642,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00245,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 133119.59646944542,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 536561.7661788866,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -20.384614944458008,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 48.380835393733605,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 49.75576798143852,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 14.38828125,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 14.91796875,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 86663.2478015409,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 104329.21754570116,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002367,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2436178.6956977653,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2409007.6127690594,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": -1.0158729553222656,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Dropped Metrics %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 16.63405103036372,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 17.736129876125258,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.548046875,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 22,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "metrics_produced_rate",
            "value": 104995.85266381978,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Metric Throughput"
          },
          {
            "name": "metrics_received_rate",
            "value": 106062.47719881732,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Metric Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00237,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 954788.8873085725,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 934294.3809477171,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": -1115,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Dropped Metrics %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 16.54841308296862,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 19.025738887595995,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 15.211328125,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 16.23046875,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "metrics_produced_rate",
            "value": 1666.6025580216015,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Metric Throughput"
          },
          {
            "name": "metrics_received_rate",
            "value": 20249.22107996246,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Metric Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002308,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 817366.2815374479,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 788692.5392363971,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Dropped Metrics %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 17.132358038961428,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 18.180733405539222,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 31.1328125,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 32.703125,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "metrics_produced_rate",
            "value": 104986.88713779648,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Metric Throughput"
          },
          {
            "name": "metrics_received_rate",
            "value": 104986.88713779648,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Metric Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.007494,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 219345.54188944862,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 922460.7483538159,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Network Utilization"
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
        "date": 1774233772984,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_signals_percentage",
            "value": -1115,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Dropped Metrics %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 16.343501474639826,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 18.86466295609153,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 15.365104166666667,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 16.50390625,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "metrics_produced_rate",
            "value": 1666.6063077415547,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Metric Throughput"
          },
          {
            "name": "metrics_received_rate",
            "value": 20249.26663905989,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Metric Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002173,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 819299.6830773327,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 788463.1994215897,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Dropped Spans %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 27.140045808197144,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 28.589135231042196,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 24.105208333333334,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 25.14453125,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "spans_produced_rate",
            "value": 104995.45719655197,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Span Throughput"
          },
          {
            "name": "spans_received_rate",
            "value": 104995.45719655197,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Span Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002596,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3638311.400569173,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3616806.4426891706,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": -5270,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Dropped Metrics %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 52.813219208399516,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 55.19851393188855,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 14.366015625,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 15.0546875,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "metrics_produced_rate",
            "value": 1666.5979472779738,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Metric Throughput"
          },
          {
            "name": "metrics_received_rate",
            "value": 89496.3097688272,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Metric Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002474,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3779224.1822506366,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3753685.4940161156,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.5873017311096191,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 14.222802424853509,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 15.276883116883116,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 31.955859375,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 33.34375,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104995.8876610666,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106662.48905251212,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00235,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 130789.1017238009,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 534148.3799057264,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -10.877192497253418,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 48.59045517119432,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 50.13328387046913,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 14.818359375,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 15.34765625,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 94996.50254543129,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 105329.4554538817,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002209,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2391280.524328083,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2365912.7174776285,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1230,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 11.489953886253756,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 12.291664216634429,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 15.507421875,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.0234375,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1666.6051411602054,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 22165.848377430735,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002215,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 492179.61040546786,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 480900.5913403502,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": -1.5873017311096191,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Dropped Metrics %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 16.701516285903192,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 18.123346957661756,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.866796875,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 21.484375,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "metrics_produced_rate",
            "value": 104996.19738771794,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Metric Throughput"
          },
          {
            "name": "metrics_received_rate",
            "value": 106662.8036954595,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Metric Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002173,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 946996.6683801241,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 924961.5445863006,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.5625,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 16.517730229208116,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 17.481294687475973,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.309505208333334,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 21.79296875,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106662.83036020138,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108329.43708457952,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002158,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 567593.9092506665,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 547095.390358588,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": -1,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Dropped Metrics %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 17.056830013671856,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 18.471571164510166,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 31.942317708333334,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 33.6640625,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "metrics_produced_rate",
            "value": 104996.20613708491,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Metric Throughput"
          },
          {
            "name": "metrics_received_rate",
            "value": 106046.16819845577,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Metric Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002168,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 220019.66749367275,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 932111.6623669612,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": -925,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Dropped Spans %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 21.282319774450823,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 22.401638098192816,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 23.128255208333332,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 25.671875,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "spans_produced_rate",
            "value": 1666.6069743602015,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Span Throughput"
          },
          {
            "name": "spans_received_rate",
            "value": 17082.721487192066,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Span Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002149,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2035760.13241674,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2048723.0514596864,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": -2766,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Dropped Spans %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 52.71785620256971,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 54.58662322550617,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.817057291666668,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.48046875,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "spans_produced_rate",
            "value": 1666.5427869861674,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Span Throughput"
          },
          {
            "name": "spans_received_rate",
            "value": 47763.11627502355,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Span Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00446,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 6116150.9667123435,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 6104760.146594777,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Dropped Spans %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 30.644311726087224,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 31.580995433083054,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 50.630078125,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 54.203125,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "spans_produced_rate",
            "value": 104995.40120142738,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Span Throughput"
          },
          {
            "name": "spans_received_rate",
            "value": 104995.40120142738,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Span Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002628,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 669630.4199682848,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3603025.614570643,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Network Utilization"
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
        "date": 1774293491431,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_signals_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Dropped Metrics %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 16.643260266428577,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 17.470700785564283,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.494010416666665,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.484375,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "metrics_produced_rate",
            "value": 104996.19913759122,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Metric Throughput"
          },
          {
            "name": "metrics_received_rate",
            "value": 104996.19913759122,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Metric Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002172,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 943773.6535376974,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 921903.5501117962,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Dropped Spans %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 30.796491688455635,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 32.71341031562741,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 50.70690104166667,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 55.71484375,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "spans_produced_rate",
            "value": 106661.9948712913,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Span Throughput"
          },
          {
            "name": "spans_received_rate",
            "value": 106661.9948712913,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Span Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002628,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 674474.3745242539,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3614991.3895869893,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1240,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 11.526112008866848,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 12.269365202545398,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.681770833333335,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.7890625,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1666.6050022815823,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 22332.5070305732,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00222,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 494782.9195203876,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 489007.6681367976,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.5873017311096191,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 16.49203857310261,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 17.48924124062186,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.1953125,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 21.4921875,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104996.16239026464,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106662.76814249106,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002193,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 567333.5539531551,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 547494.7982706252,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": -939.9999389648438,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Dropped Spans %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 21.298707935837253,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 22.232738030362007,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 23.838541666666668,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 26.19140625,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "spans_produced_rate",
            "value": 1666.6063632930882,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Span Throughput"
          },
          {
            "name": "spans_received_rate",
            "value": 17332.706178248118,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Span Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002171,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2024858.6461874773,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2034354.0378460572,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": -2746,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Dropped Spans %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 51.48021767506058,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 54.428164187242004,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.44375,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.3125,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "spans_produced_rate",
            "value": 1666.6048634029821,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Span Throughput"
          },
          {
            "name": "spans_received_rate",
            "value": 47431.57441244887,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Span Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002225,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 5984248.346546557,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5951145.601494214,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": -1.5625,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Dropped Spans %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 27.448828332542824,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 29.419232602845057,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 24.635286458333333,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 25.94921875,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "spans_produced_rate",
            "value": 106662.87657911888,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Span Throughput"
          },
          {
            "name": "spans_received_rate",
            "value": 108329.48402566761,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Span Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002132,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3665637.093795327,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3627769.2823568373,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -122.14285278320312,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 48.33780729550797,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 49.4771249516441,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 14.733463541666667,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 15.47265625,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 46664.87473547682,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 103662.6860195235,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002304,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2432071.6976519087,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2406892.321596241,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": -1,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Dropped Metrics %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 17.040982292203743,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 18.511503961843218,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 30.776692708333332,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 33.640625,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "metrics_produced_rate",
            "value": 104996.23763481808,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Metric Throughput"
          },
          {
            "name": "metrics_received_rate",
            "value": 106046.20001116626,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Metric Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00215,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 216483.75697793285,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 926299.4139539286,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": -5106,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Dropped Metrics %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 51.87139900360115,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 54.09189581884226,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 14.209765625,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 14.875,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "metrics_produced_rate",
            "value": 1666.606613275035,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Metric Throughput"
          },
          {
            "name": "metrics_received_rate",
            "value": 86763.54028709832,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Metric Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002162,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3658886.638043081,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3632785.339492539,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.5625,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 14.000735216164173,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 15.021948463978951,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 29.717447916666668,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 31.2421875,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106662.68281546351,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108329.28723445513,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002241,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 132573.36112331168,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 533125.2040160734,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": -1115,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Dropped Metrics %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 16.255889470430773,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 18.828178137651822,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 15.117447916666666,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 16.234375,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "metrics_produced_rate",
            "value": 1666.6075298761484,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Metric Throughput"
          },
          {
            "name": "metrics_received_rate",
            "value": 20249.281487995202,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Metric Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002129,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 816717.3347750976,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 789210.5360274588,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Network Utilization"
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
        "date": 1774317259467,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_signals_percentage",
            "value": -2781,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Dropped Spans %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 52.653590608869024,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 54.23213683151459,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.173958333333335,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.80859375,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "spans_produced_rate",
            "value": 1666.6059744324311,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Span Throughput"
          },
          {
            "name": "spans_received_rate",
            "value": 48014.91812339834,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Span Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002185,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 6124453.056489143,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 6097045.970910342,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": -930,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Dropped Spans %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 21.579310822391935,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 23.168367331109906,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 23.079947916666665,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 26.15625,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "spans_produced_rate",
            "value": 1666.600335973295,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Span Throughput"
          },
          {
            "name": "spans_received_rate",
            "value": 17165.983460524938,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Span Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002388,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2031323.1180588454,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2041044.8735289688,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": -3.1746034622192383,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Dropped Spans %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 27.28245495629114,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 29.08704318936877,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 23.890755208333335,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 24.88671875,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "spans_produced_rate",
            "value": 104996.14839128984,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Span Throughput"
          },
          {
            "name": "spans_received_rate",
            "value": 108329.35945133079,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Span Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002201,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3670701.1367959958,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3630767.9399516,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": -1.5625,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Dropped Spans %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 30.917578754384866,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 32.916689868522816,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 49.888020833333336,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 53.078125,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "spans_produced_rate",
            "value": 106662.07841959332,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Span Throughput"
          },
          {
            "name": "spans_received_rate",
            "value": 108328.67339489947,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Span Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002581,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 673578.9471817286,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3614209.7381451475,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": -1.5625,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Dropped Metrics %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 16.286236262817084,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 16.55786468032278,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.634244791666667,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.68359375,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "metrics_produced_rate",
            "value": 106662.77703073095,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Metric Throughput"
          },
          {
            "name": "metrics_received_rate",
            "value": 108329.38292183612,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Metric Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002188,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 939944.0933297494,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 918840.2166653621,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.5714285373687744,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 14.227596865051007,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 15.568312567297337,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 30.247526041666667,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 32.65625,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104995.97340441993,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106645.91012934654,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002301,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 133835.5057011236,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 542469.6357323828,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -67.83783721923828,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 48.55776182611146,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 50.08263715855451,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 15.141927083333334,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 15.78125,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 61664.408721567306,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 103496.21031376567,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002197,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2447012.1536601367,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2423928.141420393,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": -1.5873017311096191,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Dropped Metrics %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 16.758532027483106,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 17.28331811498878,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 30.648828125,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 33.9453125,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "metrics_produced_rate",
            "value": 104996.16938975343,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Metric Throughput"
          },
          {
            "name": "metrics_received_rate",
            "value": 106662.77525308284,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Metric Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002189,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 214537.85637643276,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 909414.0658379016,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.0158729553222656,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 16.651088654689445,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 18.07105283455457,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.517578125,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.96484375,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104996.07664660264,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106062.70345698082,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002242,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 566003.6615661792,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 546269.9298281608,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": -1115,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Dropped Metrics %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 16.234315368268557,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 18.852857914640737,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 15.335416666666667,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 16.65234375,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "metrics_produced_rate",
            "value": 1666.6061966384984,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Metric Throughput"
          },
          {
            "name": "metrics_received_rate",
            "value": 20249.265289157756,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Metric Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002177,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 818206.8794323978,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 791461.8685417383,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": -5071,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Dropped Metrics %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 51.04558415477872,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 54.19079112210966,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 14.120182291666667,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 14.515625,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "metrics_produced_rate",
            "value": 1666.6061133112162,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Metric Throughput"
          },
          {
            "name": "metrics_received_rate",
            "value": 86180.20211932299,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Metric Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00218,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3571855.0605276744,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3546685.66256969,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1230,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 11.473840635137273,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 12.23743382123949,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 15.513411458333334,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 16.1640625,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1666.6050022815823,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 22165.846530345043,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00222,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 486581.07192566834,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 481879.6708535879,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Network Utilization"
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
        "date": 1774373939998,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_signals_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Dropped Metrics %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 16.975312179747004,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 18.272505198305737,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 31.566015625,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 35.4296875,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "metrics_produced_rate",
            "value": 104996.06614738834,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Metric Throughput"
          },
          {
            "name": "metrics_received_rate",
            "value": 104996.06614738834,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Metric Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002248,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 216001.98080990958,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 904045.5608856961,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": -4156,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Dropped Metrics %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 47.559507761120415,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 48.53428483543525,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 14.766145833333333,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 15.46484375,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "metrics_produced_rate",
            "value": 1666.605307814584,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Metric Throughput"
          },
          {
            "name": "metrics_received_rate",
            "value": 70930.72190058869,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Metric Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002209,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2969116.8273075195,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2942574.9743212787,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Dropped Spans %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 30.783671860172973,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 32.19499808326305,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 50.43971354166667,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 54.140625,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "spans_produced_rate",
            "value": 104993.25418341871,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Span Throughput"
          },
          {
            "name": "spans_received_rate",
            "value": 104993.25418341871,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Span Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003855,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 668700.058700746,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3618747.7401459673,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -120.00000762939453,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 48.386447013203174,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 49.641808049535605,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 14.52109375,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 15.1796875,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 46664.8724023228,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 102662.71928511016,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002307,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2417190.7412388814,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2393755.4075591634,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1230,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 11.446805014289655,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 12.464648715567936,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 14.579947916666667,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 15.05078125,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1666.6004470755695,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 22165.785946105076,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002384,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 485478.07032966043,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 478899.4268545404,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": -2731,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Dropped Spans %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 52.633873217070594,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 54.073787855257024,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.119401041666666,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.90234375,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "spans_produced_rate",
            "value": 1666.6006692801632,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Span Throughput"
          },
          {
            "name": "spans_received_rate",
            "value": 47181.46494732142,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Span Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002376,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 6118509.490443745,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 6081945.695292407,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Dropped Metrics %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 16.690341652693647,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 17.872413420748167,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.679817708333335,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.8203125,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "metrics_produced_rate",
            "value": 104995.72317420937,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Metric Throughput"
          },
          {
            "name": "metrics_received_rate",
            "value": 104995.72317420937,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Metric Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002444,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 950102.9434569483,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 929263.8217360127,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 14.335744754129514,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 15.891507942598418,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 31.890755208333335,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 34.6796875,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104995.99090308069,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 104995.99090308069,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002291,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 133223.2581871828,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 538875.8956494677,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Dropped Spans %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 27.0933404990749,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 28.255236034922355,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 24.943359375,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 25.7421875,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "spans_produced_rate",
            "value": 104995.57618639001,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Span Throughput"
          },
          {
            "name": "spans_received_rate",
            "value": 104995.57618639001,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Span Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002528,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3621606.4012439577,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3615560.1126462133,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": -1115,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Dropped Metrics %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 16.301974920330338,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 18.988157118459867,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 13.808072916666667,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 14.8828125,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "metrics_produced_rate",
            "value": 1666.6029746563186,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Metric Throughput"
          },
          {
            "name": "metrics_received_rate",
            "value": 20249.22614207427,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Metric Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002293,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 816572.1625449011,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 788016.1034210328,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.0158729553222656,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 16.452019240385425,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 17.55905283455457,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.953645833333333,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 21.30859375,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104996.12389309295,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106062.75118343548,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002215,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 567065.1857836762,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 547238.4181022741,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": -914.9999389648438,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Dropped Spans %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 21.305316867183212,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 23.390776818742292,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 24.700260416666666,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 28.2109375,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "spans_produced_rate",
            "value": 1666.6044467673207,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Span Throughput"
          },
          {
            "name": "spans_received_rate",
            "value": 16916.035134688304,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Span Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00224,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2020490.8188825666,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2026959.7529150858,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Network Utilization"
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
        "date": 1774403982901,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_signals_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Dropped Metrics %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 16.473661267849746,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 17.812497310588597,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.211067708333335,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.2578125,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "metrics_produced_rate",
            "value": 104995.74242264476,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Metric Throughput"
          },
          {
            "name": "metrics_received_rate",
            "value": 104995.74242264476,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Metric Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002433,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 950030.9390733017,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 928957.6836741007,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": -2741,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Dropped Spans %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 52.0809295154098,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 54.206069605568445,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.732161458333334,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.6953125,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "spans_produced_rate",
            "value": 1666.6060855354574,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Span Throughput"
          },
          {
            "name": "spans_received_rate",
            "value": 47348.27889006235,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Span Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002181,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 6110140.866189267,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 6078084.335915083,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": -1.5714285373687744,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Dropped Metrics %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 17.27821669537755,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 18.19435635914078,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 30.654947916666668,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 32.87109375,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "metrics_produced_rate",
            "value": 104996.16064039258,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Metric Throughput"
          },
          {
            "name": "metrics_received_rate",
            "value": 106646.10030759875,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Metric Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002194,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 217999.28532701128,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 920245.2241416847,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.5873017311096191,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 16.541034859716945,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 18.07459509486349,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.753385416666667,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 21.890625,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104996.07664660264,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106662.68103781855,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002242,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 563734.3292608393,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 543589.546920084,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": -925,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Dropped Spans %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 21.19411413662187,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 22.12017273288094,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 24.466927083333335,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 26.8515625,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "spans_produced_rate",
            "value": 1666.6058077779192,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Span Throughput"
          },
          {
            "name": "spans_received_rate",
            "value": 17082.709529723674,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Span Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002191,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2027893.4158263677,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2035821.179360537,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": -1.5625,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Dropped Spans %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 30.583332063118746,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 32.34087117988395,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 49.92200520833333,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 53.66796875,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "spans_produced_rate",
            "value": 106662.78769662077,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Span Throughput"
          },
          {
            "name": "spans_received_rate",
            "value": 108329.39375438046,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Span Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002182,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 672988.0570459908,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3621413.087448952,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 14.237508317603002,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 15.674952681388014,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 29.81640625,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 32.82421875,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104996.01890095002,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 104996.01890095002,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002275,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 132454.6915720906,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 538734.0235947303,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": -4146,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Dropped Metrics %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 47.59116598787128,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 48.548698334361504,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 14.490755208333333,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 15.07421875,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "metrics_produced_rate",
            "value": 1666.6049467301393,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Metric Throughput"
          },
          {
            "name": "metrics_received_rate",
            "value": 70764.04603816172,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Metric Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002222,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2969168.0007884377,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2943434.2430530945,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -224.2105255126953,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 48.75175049122893,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 49.57403026134801,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 14.698697916666667,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 15.234375,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 31665.356776408014,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 102662.41986456493,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002482,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2434279.287626997,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2406932.7539120326,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": -1.5625,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Dropped Spans %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 27.288830848225448,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 28.707973204549,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 25.160807291666668,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 26.140625,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "spans_produced_rate",
            "value": 106662.83213785132,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Span Throughput"
          },
          {
            "name": "spans_received_rate",
            "value": 108329.43889000524,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Span Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002157,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3662301.0369198755,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3625150.4462657096,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": -1125,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Dropped Metrics %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 16.371821088314764,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 19.048135593220337,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 15.647916666666667,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 16.76953125,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "metrics_produced_rate",
            "value": 1666.606335517321,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Metric Throughput"
          },
          {
            "name": "metrics_received_rate",
            "value": 20415.92761008718,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Metric Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002172,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 812065.259163575,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 785316.4920089243,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1230,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 11.677658094467832,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 12.382915798410126,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 15.502083333333333,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 16.54296875,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1666.606779929708,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 22165.870173065116,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002156,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 493258.17800778995,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 479583.37267769186,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Network Utilization"
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
        "date": 1774460519432,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.5873017311096191,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 14.238507831903846,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 15.223836103594898,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 30.593489583333334,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 32.6796875,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104995.7774198181,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106662.37706140251,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002413,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 130080.91299194543,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 527207.3560999385,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": -1115,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Dropped Metrics %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 16.60170405763421,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 18.913815901525123,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 14.763671875,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 16.0546875,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "metrics_produced_rate",
            "value": 1666.6041412346347,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Metric Throughput"
          },
          {
            "name": "metrics_received_rate",
            "value": 20249.24031600081,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Metric Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002251,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 819931.7458434391,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 791773.2352858197,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.0158729553222656,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 16.618717731221437,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 17.61333128362798,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.448046875,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 21.6640625,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104996.0731468643,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106062.6999216896,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002244,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 566044.1483048435,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 545898.1289388052,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": -1,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Dropped Metrics %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 17.051370553708008,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 18.165864522115683,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 30.380208333333332,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 32.578125,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "metrics_produced_rate",
            "value": 104995.93315752236,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Metric Throughput"
          },
          {
            "name": "metrics_received_rate",
            "value": 106045.89248909759,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Metric Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002324,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 217699.21500273194,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 922037.5627012292,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1230,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 11.594672092322494,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 12.212788610337357,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 14.721354166666666,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 16.64453125,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1666.604780075833,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 22165.84357500858,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002228,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 484940.75183684373,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 480934.1937340886,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": -4166,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Dropped Metrics %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 47.53899773944653,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 48.644345679012346,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 15.325911458333334,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 16.23046875,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "metrics_produced_rate",
            "value": 1666.602058060216,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Metric Throughput"
          },
          {
            "name": "metrics_received_rate",
            "value": 71097.24379684881,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Metric Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002326,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2964219.905857821,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2934682.747406465,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": -1.0158729553222656,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Dropped Metrics %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 16.589886381853443,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 17.86527807733785,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.504036458333335,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 21.57421875,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "metrics_produced_rate",
            "value": 104995.87716189011,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Metric Throughput"
          },
          {
            "name": "metrics_received_rate",
            "value": 106062.50194575693,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Metric Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002356,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 962228.9800735188,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 941000.8603357635,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Dropped Spans %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 30.98098968793732,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 32.285643564356434,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 50.04635416666667,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 55.4609375,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "spans_produced_rate",
            "value": 104995.76867052258,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Span Throughput"
          },
          {
            "name": "spans_received_rate",
            "value": 104995.76867052258,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Span Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002418,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 668647.6178698663,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3630278.8571949084,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": -910.0000610351562,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Dropped Spans %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 21.28589693945866,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 22.120146406043144,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 24.176171875,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 26.671875,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "spans_produced_rate",
            "value": 1666.6040856832492,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Span Throughput"
          },
          {
            "name": "spans_received_rate",
            "value": 16832.701265400818,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Span Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002253,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2040773.5859844286,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2037967.440742602,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": -1.5625,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Dropped Spans %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 27.562453477104942,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 29.028132548776707,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 25.127083333333335,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 26.8359375,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "spans_produced_rate",
            "value": 106662.68103781855,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Span Throughput"
          },
          {
            "name": "spans_received_rate",
            "value": 108329.28542903447,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Span Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002242,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3675893.0464157932,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3651877.9246491273,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": -2516,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Dropped Spans %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 48.23149854059412,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 53.560402414486916,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.872135416666666,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 22.48046875,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "spans_produced_rate",
            "value": 1666.6001970954726,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Span Throughput"
          },
          {
            "name": "spans_received_rate",
            "value": 43598.261156017565,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Span Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002393,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 5617304.623865757,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5584929.765837567,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -16.38888931274414,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 48.270732521307494,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 49.51198885621421,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 15.091536458333334,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 16.19921875,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 89996.5606314412,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 104745.99695714962,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002293,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2438348.8883099156,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2413279.726369329,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Network Utilization"
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
        "date": 1774490525229,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_signals_percentage",
            "value": -0.4285714030265808,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Dropped Spans %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 30.814407691743156,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 32.432245087420704,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 49.99075520833333,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 54.29296875,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "spans_produced_rate",
            "value": 104996.0783964719,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Span Throughput"
          },
          {
            "name": "spans_received_rate",
            "value": 105446.06158959963,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Span Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002241,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 671649.2018666117,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3642721.589012142,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": -4161,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Dropped Metrics %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 47.673344731441716,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 48.53064384301026,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 14.21171875,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 14.65625,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "metrics_produced_rate",
            "value": 1666.6033357399085,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Metric Throughput"
          },
          {
            "name": "metrics_received_rate",
            "value": 71013.9681358775,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Metric Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00228,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2967251.0872622277,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2941156.946541621,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": -930,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Dropped Spans %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 21.295018758805494,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 22.559789555125725,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 23.433463541666665,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 27.01953125,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "spans_produced_rate",
            "value": 1666.6057522264227,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Span Throughput"
          },
          {
            "name": "spans_received_rate",
            "value": 17166.039247932156,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Span Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002193,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2022607.2166805556,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2030990.6821893067,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": -0.4761905074119568,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Dropped Spans %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 27.179337283219922,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 28.253361207897793,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 24.212239583333332,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 25.17578125,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "spans_produced_rate",
            "value": 104996.0696471262,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Span Throughput"
          },
          {
            "name": "spans_received_rate",
            "value": 105496.05093116015,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Span Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002246,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3643378.0279720277,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3639608.365347472,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -237.5,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 48.58955541972477,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 49.13210188124177,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 14.90390625,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 15.60546875,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 29998.334092513396,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 101244.37756223271,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003332,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2422711.032087012,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2397661.9680350693,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": -1125,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Dropped Metrics %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 16.532949684078694,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 19.282076637824474,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 14.3296875,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 15.2890625,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "metrics_produced_rate",
            "value": 1666.6044189916174,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Metric Throughput"
          },
          {
            "name": "metrics_received_rate",
            "value": 20415.904132647313,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Metric Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002241,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 816864.3153219362,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 788876.889474867,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1225,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 11.518557021269023,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 12.441892978657593,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.284505208333332,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.45703125,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1666.6019191821065,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 22082.47542916291,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002331,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 494238.0458739332,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 490301.3491166724,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Dropped Metrics %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 17.079740340904856,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 18.22464962775347,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 30.738671875,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 32.56640625,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "metrics_produced_rate",
            "value": 104995.16672249188,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Metric Throughput"
          },
          {
            "name": "metrics_received_rate",
            "value": 104995.16672249188,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Metric Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002762,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 217717.4315558185,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 921599.0542607821,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.5714285373687744,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 14.304139841020373,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 15.439598796389168,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 30.5265625,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 32.4453125,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104995.59368491835,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106645.52444282421,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002518,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 132891.11664092357,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 541831.2251993318,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": -2746,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Dropped Spans %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 52.676072580968736,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 54.16937026148847,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.709895833333334,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.82421875,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "spans_produced_rate",
            "value": 1666.6048911787004,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Span Throughput"
          },
          {
            "name": "spans_received_rate",
            "value": 47431.57520294581,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Span Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002224,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 6143580.960317543,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 6107436.938903579,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.5873017311096191,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 16.581544915177354,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 17.69180126134441,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.503385416666667,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 21.72265625,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104995.59193506525,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106662.18863244724,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002519,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 566138.1380991639,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 546267.2673743461,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Dropped Metrics %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 16.713797338866925,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 17.88734091607747,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.670703125,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.76953125,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "metrics_produced_rate",
            "value": 104996.02065081733,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Metric Throughput"
          },
          {
            "name": "metrics_received_rate",
            "value": 104996.02065081733,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Metric Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002274,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 959123.7215227045,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 937144.2839193658,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Network Utilization"
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
        "date": 1775494202643,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -95.23809814453125,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 14.35436152000894,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 15.365940594059404,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 29.573307291666666,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 32.04296875,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104990.0696892419,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 204980.61225042466,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.005675,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 142069.81808729417,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 537410.4373841411,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Network Utilization"
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
        "date": 1775524974274,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -96.23809814453125,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 14.606357361500082,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 15.827072454812296,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 30.373697916666668,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 31.78515625,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104994.10283122431,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 206038.42750832162,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00337,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 143038.76031461696,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 536212.6496360246,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Network Utilization"
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
        "date": 1775585054330,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -95.23809814453125,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 14.51170308903553,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 16.013017115665054,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 29.734244791666665,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 31.92578125,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104993.57439324712,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 204987.4547677682,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003672,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 145141.09260401566,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 540995.1944569588,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Network Utilization"
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
        "date": 1775612563674,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -95.23809814453125,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 14.565800065334455,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 15.514755720470005,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 28.9640625,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 30.00390625,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104995.09847881935,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 204990.43036340922,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002801,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 142989.82304806283,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 538146.5021733505,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Network Utilization"
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
        "date": 1775673349714,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -96.23809814453125,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 14.414662229653901,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 15.158482811896484,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 29.688151041666668,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 31.890625,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104993.54464689663,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 206037.33213802904,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003689,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 141962.42438638525,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 540827.8506650596,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Network Utilization"
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
        "date": 1775699586744,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -95.23809814453125,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 14.395706347400658,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 15.39892824145365,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 31.973177083333333,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 33.84765625,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104995.25771419324,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 204990.74125152014,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00271,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 144961.73493722753,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 539719.2945812062,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Network Utilization"
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
        "date": 1775764205037,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_signals_percentage",
            "value": -2771,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Dropped Spans %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 52.87589205788935,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 54.31299093655589,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 21.937630208333335,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.06640625,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "spans_produced_rate",
            "value": 1666.6013636699004,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Span Throughput"
          },
          {
            "name": "spans_received_rate",
            "value": 47848.125150962835,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Span Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002351,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 6193546.941114997,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 6130668.328949619,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Dropped Spans %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 30.8865779127942,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 31.823055620020114,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 50.514713541666666,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 55.12109375,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "spans_produced_rate",
            "value": 104995.62693213827,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Span Throughput"
          },
          {
            "name": "spans_received_rate",
            "value": 104995.62693213827,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Span Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002499,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 681878.5763053219,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3616113.501489865,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Dropped Metrics %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 16.911889660080064,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 17.530676598386098,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 31.239322916666666,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 32.1484375,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "metrics_produced_rate",
            "value": 104995.90515969878,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Metric Throughput"
          },
          {
            "name": "metrics_received_rate",
            "value": 104995.90515969878,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Metric Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00234,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 226842.75015341977,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 901661.5365455443,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.5714285373687744,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 14.426878263580162,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 15.417575804336083,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 31.575,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 33.48046875,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104984.59351090227,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106634.35140893073,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.008805,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 144729.95885498458,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 539605.2508324936,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -589,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 48.37944751529869,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 49.411346793532914,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.3671875,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.984375,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 14999.300032665142,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 103345.17722506283,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.0028,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2423952.4216732993,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2384862.8425722676,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": -939.9999389648438,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Dropped Spans %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 21.27692011214454,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 22.221017788089714,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 26.152604166666666,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 29.27734375,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "spans_produced_rate",
            "value": 1666.5956141403171,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Span Throughput"
          },
          {
            "name": "spans_received_rate",
            "value": 17332.594387059296,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Span Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002558,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2062440.2575182288,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2040669.9173330064,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": -1125,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Dropped Metrics %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 16.29771299982187,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 18.723572706761068,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.423307291666667,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.5859375,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "metrics_produced_rate",
            "value": 1666.6042801131146,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Metric Throughput"
          },
          {
            "name": "metrics_received_rate",
            "value": 20415.902431385653,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Metric Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002246,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 819824.2607647697,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 780027.0361179473,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": -2605.5,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Dropped Metrics %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 53.016585087012636,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 54.64728961833243,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.613151041666665,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.09375,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "metrics_produced_rate",
            "value": 3332.9257165182034,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Metric Throughput"
          },
          {
            "name": "metrics_received_rate",
            "value": 90172.3052604,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Metric Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.007338,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3780862.2485342445,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3740446.8195133237,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": -1.5873017311096191,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Dropped Spans %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 28.148586639128208,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 29.483233884680786,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 27.210677083333334,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 28.296875,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "spans_produced_rate",
            "value": 104995.90515969878,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Span Throughput"
          },
          {
            "name": "spans_received_rate",
            "value": 106662.50682890034,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Span Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00234,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3676146.494785609,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3659720.050670534,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Dropped Metrics %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 17.734151113468183,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 19.809848099313747,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 21.901041666666668,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.32421875,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "metrics_produced_rate",
            "value": 104995.96640495727,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Metric Throughput"
          },
          {
            "name": "metrics_received_rate",
            "value": 104995.96640495727,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Metric Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002305,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 964679.9161196743,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 932555.8708831166,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.0158729553222656,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 17.024630702479225,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 17.787394970986462,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 23.254427083333333,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 24.40625,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104992.87273382259,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106059.46699651539,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004073,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 570878.7507150759,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 538990.9121409578,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1235,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 11.669580015276543,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 12.499616099071208,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.5453125,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.40625,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1666.6028080024066,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 22249.147486832127,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002299,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 508578.84355614096,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 492241.0688695976,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Network Utilization"
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
        "date": 1775790500033,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_signals_percentage",
            "value": -1.5714285373687744,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Dropped Metrics %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 17.198898341778843,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 18.51937053811315,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 30.743098958333334,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 33.84765625,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "metrics_produced_rate",
            "value": 104995.96990468849,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Metric Throughput"
          },
          {
            "name": "metrics_received_rate",
            "value": 106645.90657461931,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Metric Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002303,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 227580.20223704615,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 915019.6893786038,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 14.356143462934575,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 15.676410058027079,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 32.56614583333333,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 34.375,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104994.17807282586,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 104994.17807282586,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003327,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 144146.5463843122,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 545409.4144224034,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": -0.4285714030265808,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Dropped Spans %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 30.825054556649693,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 32.05855108359133,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 48.887369791666664,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 53.3984375,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "spans_produced_rate",
            "value": 104995.86666271572,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Span Throughput"
          },
          {
            "name": "spans_received_rate",
            "value": 105445.84894841307,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Span Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002362,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 683520.6029453772,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3636961.8153077164,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Dropped Spans %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 27.86727621349534,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 28.859754421190825,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 28.40234375,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 29.45703125,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "spans_produced_rate",
            "value": 104992.08009742465,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Span Throughput"
          },
          {
            "name": "spans_received_rate",
            "value": 104992.08009742465,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Span Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004526,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3660524.561771472,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3645393.8731746823,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.5873017311096191,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 17.062578628480853,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 18.203024533704824,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 22.758072916666666,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.75,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104993.48515424618,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106660.04841066278,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003723,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 576597.5459247808,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 544992.3247838778,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": -2655,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Dropped Metrics %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 53.216556736902234,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 55.127155679356235,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.390494791666665,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.0703125,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "metrics_produced_rate",
            "value": 3333.024861882366,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Metric Throughput"
          },
          {
            "name": "metrics_received_rate",
            "value": 91824.83494485919,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Metric Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.005553,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3817907.1081731594,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3783490.0363667216,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": -2791,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Dropped Spans %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 52.71532952727092,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 53.90588927603561,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.617447916666666,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 21.35546875,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "spans_produced_rate",
            "value": 1666.5968084837777,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Span Throughput"
          },
          {
            "name": "spans_received_rate",
            "value": 48181.31373326601,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Span Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002515,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 6197583.491334655,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 6173725.768815434,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1245,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 11.603475015902557,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 12.486860150143178,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.240755208333333,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.51171875,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1666.604335664513,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 22415.828314687697,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002244,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 501590.32463295973,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 475563.6722748123,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": -1125,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Dropped Metrics %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 16.337714455498777,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 18.824958715950306,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.230338541666665,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.61328125,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "metrics_produced_rate",
            "value": 1666.6047245244054,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Metric Throughput"
          },
          {
            "name": "metrics_received_rate",
            "value": 20415.907875423964,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Metric Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00223,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 818650.3694761846,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 779959.0520216431,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": -930,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Dropped Spans %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 21.40225841061373,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 22.620267781131492,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 26.545572916666668,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 28.6484375,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "spans_produced_rate",
            "value": 1666.603919029115,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Span Throughput"
          },
          {
            "name": "spans_received_rate",
            "value": 17166.020365999888,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Span Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002259,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2056353.450893521,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2046961.125829769,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -213.8000030517578,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 48.59607502044618,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 49.72597106802093,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.384765625,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.02734375,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 33329.83592255053,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 104589.02512496356,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.006296,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2448133.2412609654,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2409384.4987715385,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": -1.5873017311096191,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Dropped Metrics %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 17.383353581775626,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 18.403465346534652,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 22.23046875,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.29296875,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "metrics_produced_rate",
            "value": 104996.69260418296,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Metric Throughput"
          },
          {
            "name": "metrics_received_rate",
            "value": 106663.30677250333,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Metric Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00189,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 959722.0882727237,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 927224.4981166802,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Network Utilization"
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
        "date": 1775842834416,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_signals_percentage",
            "value": -5306,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Dropped Metrics %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 52.706388128121574,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 54.476827810972296,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.111197916666665,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.671875,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "metrics_produced_rate",
            "value": 1666.602058060216,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Metric Throughput"
          },
          {
            "name": "metrics_received_rate",
            "value": 90096.50725873528,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Metric Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002326,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3749511.953775933,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3712178.5400649626,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1255,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 11.570420615028391,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 12.18094473602964,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.261458333333334,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 21.41015625,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1666.6028913293585,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 22582.46917751281,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002296,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 498374.28230929055,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 481287.23196379637,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Dropped Spans %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 28.113461151101387,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 29.223372452130945,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 27.180208333333333,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 28.91796875,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "spans_produced_rate",
            "value": 106662.5406040543,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Span Throughput"
          },
          {
            "name": "spans_received_rate",
            "value": 106662.5406040543,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Span Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002321,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3709432.506604717,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3675291.230210603,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": -935.0000610351562,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Dropped Spans %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 21.49215997726346,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 23.01884215427596,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 27.43203125,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 31,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "spans_produced_rate",
            "value": 1666.6023358165041,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Span Throughput"
          },
          {
            "name": "spans_received_rate",
            "value": 17249.334175700817,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Span Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002316,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2062516.2254879267,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2050247.8471803956,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.5873017311096191,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 17.036786218241527,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 18.056273387969693,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 22.385416666666668,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.6171875,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104991.48694026726,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106658.01847900165,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004865,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 578827.0365876007,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 547450.8919392513,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -6005,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 48.91863076020533,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 49.79192075530104,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.869661458333333,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.34765625,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1666.6193902299638,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 101747.11377353928,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001702,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2420371.517829975,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2381078.006349158,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": 0.015625,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Dropped Spans %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 31.27901466124371,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 33.47674806800619,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 51.971875,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 56.44140625,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "spans_produced_rate",
            "value": 106653.45408126357,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Span Throughput"
          },
          {
            "name": "spans_received_rate",
            "value": 106636.78947906337,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Span Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.007433,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 691242.7795763962,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3638793.116536344,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Dropped Metrics %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 17.644099628319392,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 18.60109820810582,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 21.5921875,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 22.65625,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "metrics_produced_rate",
            "value": 104995.9489063047,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Metric Throughput"
          },
          {
            "name": "metrics_received_rate",
            "value": 104995.9489063047,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Metric Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002315,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 969025.5253789371,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 941369.5538016804,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": -1120,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Dropped Metrics %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 16.40117778395064,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 18.803595018949647,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.900520833333335,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.23828125,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "metrics_produced_rate",
            "value": 1666.6028913293585,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Metric Throughput"
          },
          {
            "name": "metrics_received_rate",
            "value": 20332.55527421817,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Metric Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002296,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 821415.3190837965,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 782155.7460736183,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": -2816,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Dropped Spans %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 52.6653253776671,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 54.205350870406185,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 21.136979166666666,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 21.6015625,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "spans_produced_rate",
            "value": 1666.5946420015548,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Span Throughput"
          },
          {
            "name": "spans_received_rate",
            "value": 48597.899760765344,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Span Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002593,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 6159861.186059738,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 6135406.955254878,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 14.142822346125266,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 14.602584234930449,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 31.768489583333334,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 33.2421875,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106661.85799456875,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106661.85799456875,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002705,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 142000.59729359674,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 532051.6271575853,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Dropped Metrics %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 17.107753674848144,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 18.174795154694852,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 31.289713541666668,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 33.61328125,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "metrics_produced_rate",
            "value": 104995.8789117527,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Metric Throughput"
          },
          {
            "name": "metrics_received_rate",
            "value": 104995.8789117527,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Metric Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002355,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 226133.90056240727,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 911775.8909461418,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Network Utilization"
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
        "date": 1775873939120,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.5873017311096191,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 14.42583902034204,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 15.693594366633135,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 32.16796875,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 34.5546875,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104995.45719655197,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106662.0517552274,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002596,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 143410.39431856433,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 541050.4470541344,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": -1110,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Dropped Metrics %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 16.375325563193726,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 19.253585080863576,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.053385416666668,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "metrics_produced_rate",
            "value": 1666.6027802267581,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Metric Throughput"
          },
          {
            "name": "metrics_received_rate",
            "value": 20165.893640743772,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Metric Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.0023,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 820711.5700099888,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 780809.9873469897,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -122.14285278320312,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 48.67990264606595,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 49.9371296723581,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.776432291666666,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.5703125,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 46664.8039632418,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 103662.52880405857,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002395,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2453831.860330915,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2417026.4920449853,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": -0.4761905074119568,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Dropped Spans %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 27.81667236291609,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 29.3821155335295,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 27.4046875,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 28.82421875,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "spans_produced_rate",
            "value": 104996.08189621057,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Span Throughput"
          },
          {
            "name": "spans_received_rate",
            "value": 105496.06323857348,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Span Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002239,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3657619.173154217,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3641297.137828657,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": -930,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Dropped Spans %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 21.403562622854828,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 22.67339661958787,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 25.769140625,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 29.54296875,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "spans_produced_rate",
            "value": 1666.6054466932583,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Span Throughput"
          },
          {
            "name": "spans_received_rate",
            "value": 17166.03610094056,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Span Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002204,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2049312.4907355313,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2039580.2599424557,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": -2796,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Dropped Spans %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 52.715546058049654,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 53.84082959294227,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.816015625,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 21.34765625,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "spans_produced_rate",
            "value": 1666.6062799657893,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Span Throughput"
          },
          {
            "name": "spans_received_rate",
            "value": 48264.91786780926,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Span Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002174,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 6175990.32628827,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 6139204.764644993,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Dropped Spans %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 30.93062013465436,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 32.36176488738044,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 50.79270833333333,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 54.171875,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "spans_produced_rate",
            "value": 104995.96815482285,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Span Throughput"
          },
          {
            "name": "spans_received_rate",
            "value": 104995.96815482285,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Span Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002304,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 683979.2395058397,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3613465.476641605,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": -1.0158729553222656,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Dropped Metrics %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 17.324021245911112,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 18.923933118576468,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 21.995182291666666,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.078125,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "metrics_produced_rate",
            "value": 104995.98565348185,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Metric Throughput"
          },
          {
            "name": "metrics_received_rate",
            "value": 106062.61153948547,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Metric Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002294,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 956338.7372352736,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 924267.2096353045,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": -5395,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Dropped Metrics %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 53.067348960310525,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 54.953785985877246,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.046744791666665,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.59765625,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "metrics_produced_rate",
            "value": 1666.4643023515512,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Metric Throughput"
          },
          {
            "name": "metrics_received_rate",
            "value": 91572.21341421774,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Metric Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.007286,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3799306.531386695,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3761968.2330422797,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Dropped Metrics %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 16.97114634869562,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 17.629005340970664,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 30.8328125,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 31.5703125,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "metrics_produced_rate",
            "value": 106662.67926017365,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Metric Throughput"
          },
          {
            "name": "metrics_received_rate",
            "value": 106662.67926017365,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Metric Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002243,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 228029.45298496672,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 910046.6748448742,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1240,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 11.575855138938046,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 12.450278637770898,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.618880208333334,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.6875,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1666.600919260402,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 22332.452318089385,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002367,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 493455.1182205081,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 473047.45096382324,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 16.591634395936907,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 17.101727019498608,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 23.365104166666665,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 24.1484375,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106658.83435293067,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106658.83435293067,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004406,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 566340.938259801,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 534858.1864725086,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Network Utilization"
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
        "date": 1775930352828,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 14.335180831541827,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 15.290505800464038,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 31.930208333333333,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 33.421875,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104995.34520636252,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 104995.34520636252,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00266,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 144554.54390119965,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 540677.3656888006,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.0158729553222656,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 17.085008277638174,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 18.215195324515534,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 22.5984375,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.828125,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104987.76192655144,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106054.30426993227,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.006994,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 580143.6723061615,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 549143.4846106058,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": -0.4761905074119568,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Dropped Spans %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 27.865713423755643,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 28.944934622823986,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 27.413802083333334,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 28.703125,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "spans_produced_rate",
            "value": 104996.12039335146,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Span Throughput"
          },
          {
            "name": "spans_received_rate",
            "value": 105496.10191903409,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Span Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002217,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3656664.9896022314,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3640491.216500974,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": -5416,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Dropped Metrics %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 52.75888480757034,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 54.37618996055379,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.822786458333333,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.44921875,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "metrics_produced_rate",
            "value": 1666.598891645073,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Metric Throughput"
          },
          {
            "name": "metrics_received_rate",
            "value": 91929.59486314224,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Metric Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00244,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3805412.799186663,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3764216.201549895,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1240,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 11.615120556086952,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 12.18045345508009,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.797395833333333,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.12109375,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1666.602641348528,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 22332.47539407028,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002305,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 507190.49489862204,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 486128.6563031635,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": -930,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Dropped Spans %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 21.578896456401374,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 23.09718791064389,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 26.633463541666668,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 29.515625,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "spans_produced_rate",
            "value": 1666.5937254146177,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Span Throughput"
          },
          {
            "name": "spans_received_rate",
            "value": 17165.915371770563,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Span Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002626,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2048646.5142035852,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2035751.3039192758,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": -1.5873017311096191,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Dropped Metrics %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 17.47328971227883,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 19.27940641015834,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 21.719270833333333,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 22.734375,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "metrics_produced_rate",
            "value": 104996.3163792337,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Metric Throughput"
          },
          {
            "name": "metrics_received_rate",
            "value": 106662.92457572947,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Metric Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002105,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 969149.6769846114,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 936113.342747899,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Dropped Spans %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 30.657289708684793,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 31.589405940594062,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 50.602864583333336,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 53.7578125,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "spans_produced_rate",
            "value": 104996.0801463412,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Span Throughput"
          },
          {
            "name": "spans_received_rate",
            "value": 104996.0801463412,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Span Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00224,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 685015.3214802401,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3609594.3674417585,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -17.5,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 48.27542952659293,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 49.10540051079638,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.422265625,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.30859375,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 89989.19829656447,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 105737.30799846325,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.007202,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2437030.241398425,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2405113.5933448356,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": -1.5714285373687744,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Dropped Metrics %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 17.22330646904679,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 18.169282856263465,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 32.07578125,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 34.671875,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "metrics_produced_rate",
            "value": 104995.77042038157,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Metric Throughput"
          },
          {
            "name": "metrics_received_rate",
            "value": 106645.70395555899,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Metric Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002417,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 228722.58967742516,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 916979.5727698293,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": -2791,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Dropped Spans %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 52.905360825567904,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 54.09804949957328,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 21.818489583333335,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.02734375,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "spans_produced_rate",
            "value": 1666.604835627265,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Span Throughput"
          },
          {
            "name": "spans_received_rate",
            "value": 48181.545797984225,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Span Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002226,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 6181820.316465483,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 6158263.62198315,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": -1110,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Dropped Metrics %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 16.418757594323814,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 18.72561950003894,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.490885416666668,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.8984375,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "metrics_produced_rate",
            "value": 1666.4622472976648,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Metric Throughput"
          },
          {
            "name": "metrics_received_rate",
            "value": 20164.193192301744,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Metric Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00736,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 818713.3985579858,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 779608.2652331481,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Network Utilization"
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
        "date": 1775959753068,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_signals_percentage",
            "value": -930,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Dropped Spans %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 21.271405159669797,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 22.28449118465821,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 25.904557291666666,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 27.4609375,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "spans_produced_rate",
            "value": 1666.6038357020607,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Span Throughput"
          },
          {
            "name": "spans_received_rate",
            "value": 17166.019507731224,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Span Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002262,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2061440.583704411,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2046941.7319571148,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.0158729553222656,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 17.12061566977764,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 18.167022736030827,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 22.579947916666665,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.98828125,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104991.5411814988,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106058.12191731085,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004834,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 578229.184997456,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 546799.0836866312,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -465,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 49.16769252755653,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 50.05057385924208,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.940364583333334,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.7265625,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 18332.55145001399,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 103578.91569257904,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002559,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2352925.2765888437,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2318452.6724574286,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": -1.5873017311096191,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Dropped Spans %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 27.66625324632815,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 28.776681364585748,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 27.5734375,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 29.046875,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "spans_produced_rate",
            "value": 104995.41345035579,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Span Throughput"
          },
          {
            "name": "spans_received_rate",
            "value": 106662.00731464714,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Span Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002621,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3666023.9232068616,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3651352.2360552265,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1240,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 11.50142570919352,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 12.465639121015165,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.670182291666666,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 21.12109375,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1666.5673670277147,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 22332.002718171378,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003575,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 507286.60596894816,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 491524.5394751208,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": -1,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Dropped Metrics %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 17.368352091963636,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 18.620217958557177,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 30.991796875,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 32.40625,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "metrics_produced_rate",
            "value": 104995.82641589998,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Metric Throughput"
          },
          {
            "name": "metrics_received_rate",
            "value": 106045.78468005898,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Metric Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002385,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 230529.58323070884,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 929409.7629670775,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.5714285373687744,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 14.260292170415081,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 15.382964967906581,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 32.18828125,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 34.5859375,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104993.04246105292,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106642.93312829803,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003976,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 144800.7732278868,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 541845.37898878,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": -5336,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Dropped Metrics %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 52.70208253563854,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 54.38101568566548,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.96640625,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.546875,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "metrics_produced_rate",
            "value": 1666.6037801506957,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Metric Throughput"
          },
          {
            "name": "metrics_received_rate",
            "value": 90596.58148899183,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Metric Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002264,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3766292.937861321,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3724921.618771472,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": -2750,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Dropped Spans %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 52.85651751631334,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 54.854215898546244,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 22.844140625,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 24.23828125,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "spans_produced_rate",
            "value": 1666.602474694683,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Span Throughput"
          },
          {
            "name": "spans_received_rate",
            "value": 47498.170528798466,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Span Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002311,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 6094839.774339373,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 6062423.608123195,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": -1110,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Dropped Metrics %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 16.458620108922396,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 18.781162988772746,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.016276041666668,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.31640625,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "metrics_produced_rate",
            "value": 1666.603502393926,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Metric Throughput"
          },
          {
            "name": "metrics_received_rate",
            "value": 20165.902378966504,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Metric Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002274,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 818068.1664800228,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 778116.390506582,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Dropped Metrics %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 17.415310762024774,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 18.473278865423154,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 22.245572916666667,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 24.00390625,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "metrics_produced_rate",
            "value": 104995.80016799328,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Metric Throughput"
          },
          {
            "name": "metrics_received_rate",
            "value": 104995.80016799328,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Metric Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.0024,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 962892.5039812035,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 930169.3950716761,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": -0.4285714030265808,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Dropped Spans %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 30.79882046754682,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 32.21427356375165,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 48.422916666666666,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 52.7265625,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "spans_produced_rate",
            "value": 104995.80716743378,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Span Throughput"
          },
          {
            "name": "spans_received_rate",
            "value": 105445.78919815135,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Span Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002396,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 683635.7893277446,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3630471.121932608,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Network Utilization"
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
        "date": 1776014937967,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_signals_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Dropped Metrics %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 17.37985328980644,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 18.277897340754485,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 22.030729166666667,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.328125,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "metrics_produced_rate",
            "value": 104987.25979602375,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Metric Throughput"
          },
          {
            "name": "metrics_received_rate",
            "value": 104987.25979602375,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Metric Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.007281,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 948782.958910223,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 921468.3068657801,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": -1125,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Dropped Metrics %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 16.34782908188108,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 18.549157577164074,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.689453125,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.53125,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "metrics_produced_rate",
            "value": 1666.592558850883,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Metric Throughput"
          },
          {
            "name": "metrics_received_rate",
            "value": 20415.75884592332,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Metric Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002668,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 819279.9570045996,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 778844.8594754594,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": -939.9999389648438,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Dropped Spans %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 21.296683481881864,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 22.495028066128413,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 25.080859375,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 28.16796875,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "spans_produced_rate",
            "value": 1666.599586033329,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Span Throughput"
          },
          {
            "name": "spans_received_rate",
            "value": 17332.63569474662,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Span Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002415,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2054719.3768114182,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2041907.2841014152,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": -2761,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Dropped Spans %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 52.882277827629665,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 54.675888604452716,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.9625,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 21.65234375,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "spans_produced_rate",
            "value": 1666.6031968615862,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Span Throughput"
          },
          {
            "name": "spans_received_rate",
            "value": 47681.51746220998,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Span Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002285,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 6159457.464261848,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 6111498.859690829,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.5873017311096191,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 17.197192995535826,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 19.028594402350393,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 22.972526041666665,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.8984375,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104984.05642129816,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106650.470015287,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.009112,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 575991.2444539306,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 544615.7169060252,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": -0.4687500298023224,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Dropped Spans %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 27.83097346627675,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 30.092632556699435,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 27.09921875,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 28.49609375,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "spans_produced_rate",
            "value": 106662.78947426927,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Span Throughput"
          },
          {
            "name": "spans_received_rate",
            "value": 107162.77129992991,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Span Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002181,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3674786.2657852117,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3622731.35940443,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1235,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 11.610093456099424,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 12.641303437039337,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.053255208333333,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.54296875,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1666.6037245993343,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 22249.159723401113,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002266,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 511956.52884696884,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 489283.4876882286,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": -1,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Dropped Metrics %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 17.272424363440887,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 18.550026230519983,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 31.6078125,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 33.54296875,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "metrics_produced_rate",
            "value": 104995.96990468849,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Metric Throughput"
          },
          {
            "name": "metrics_received_rate",
            "value": 106045.92960373538,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Metric Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002303,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 229219.7329493191,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 920020.2529102333,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": -1.5714285373687744,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Dropped Spans %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 30.984314921127204,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 32.632946435452794,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 49.012760416666666,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 52.40234375,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "spans_produced_rate",
            "value": 104995.9279079293,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Span Throughput"
          },
          {
            "name": "spans_received_rate",
            "value": 106645.86391791105,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Span Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002327,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 686251.9407345026,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3640758.188836299,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.5714285373687744,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 14.380095704017313,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 15.438464029609067,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 31.864453125,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 34.09375,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104994.0065921237,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106643.91240999993,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003425,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 143909.37882567497,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 541171.638631535,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -141.53846740722656,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 48.40759274903993,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 49.69348286797123,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.991276041666666,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.7578125,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 43331.72283763454,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 104662.77670013264,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00223,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2478190.756215536,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2443133.7991754017,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": -4931,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Dropped Metrics %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 50.746203108475896,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 54.878786189812665,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.402213541666665,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.171875,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "metrics_produced_rate",
            "value": 1666.6037523750144,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Metric Throughput"
          },
          {
            "name": "metrics_received_rate",
            "value": 83846.83478198698,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Metric Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002265,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3476517.3041794607,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3441536.040517403,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Network Utilization"
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
        "date": 1776046220999,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_signals_percentage",
            "value": -939.9999389648438,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Dropped Spans %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 21.346121104963867,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 22.52746393671475,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 26.465625,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 29.0234375,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "spans_produced_rate",
            "value": 1666.5945309000542,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Span Throughput"
          },
          {
            "name": "spans_received_rate",
            "value": 17332.583121360563,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Span Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002597,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2058503.6241670526,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2035069.8061900917,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Dropped Metrics %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 17.04586375894189,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 18.1319801980198,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 30.810546875,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 32.234375,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "metrics_produced_rate",
            "value": 106653.6193738966,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Metric Throughput"
          },
          {
            "name": "metrics_received_rate",
            "value": 106653.6193738966,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Metric Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00734,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 228181.3970306349,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 902230.1393033091,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.5873017311096191,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 17.029735997325204,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 18.110590047758436,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 22.790364583333332,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.5234375,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104991.38195739767,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106657.9118297373,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004925,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 569792.5281175785,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 544061.9058927209,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": -5391,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Dropped Metrics %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 53.116168285004775,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 54.818419669089224,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.88671875,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.515625,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "metrics_produced_rate",
            "value": 1666.6041690103289,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Metric Throughput"
          },
          {
            "name": "metrics_received_rate",
            "value": 91513.23492035716,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Metric Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00225,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3793294.2509002145,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3759760.1028058007,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": -1125,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Dropped Metrics %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 16.206038024338834,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 18.690615277455304,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.198567708333332,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.4921875,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "metrics_produced_rate",
            "value": 1666.6008081580644,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Metric Throughput"
          },
          {
            "name": "metrics_received_rate",
            "value": 20415.859899936288,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Metric Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002371,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 820516.5269122458,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 780464.4489994866,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Dropped Metrics %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 17.23021707071416,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 18.61775923350332,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 22.551302083333333,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.12109375,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "metrics_produced_rate",
            "value": 106662.41439174625,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Metric Throughput"
          },
          {
            "name": "metrics_received_rate",
            "value": 106662.41439174625,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Metric Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002392,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 942067.6257310449,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 903956.6137643899,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -24.019607543945312,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 48.24584974670287,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 49.60750427084951,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.711979166666666,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.328125,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 84996.5689718325,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 105412.41151898835,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002422,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2449502.308501534,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2411838.563653298,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": -2.0158729553222656,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Dropped Spans %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 30.974915462641594,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 33.54195302843016,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 50.669270833333336,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 54.5234375,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "spans_produced_rate",
            "value": 104996.05214843921,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Span Throughput"
          },
          {
            "name": "spans_received_rate",
            "value": 107112.63923143156,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Span Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002256,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 689773.6484145202,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3622736.760455847,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.5714285373687744,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 14.518357921821066,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 15.325798131129817,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 32.869140625,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 34.3046875,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104994.37405145708,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106644.28564369427,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003215,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 144484.01313168817,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 534452.7119128545,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1240,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 11.54533404754323,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 12.705863428969144,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.484765625,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.7734375,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1666.5945031246813,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 22332.36634187073,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002598,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 509287.158397053,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 488781.3188226219,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Dropped Spans %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 27.906578277874072,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 30.16205381235063,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 27.187630208333335,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 29.296875,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "spans_produced_rate",
            "value": 106656.10948942904,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Span Throughput"
          },
          {
            "name": "spans_received_rate",
            "value": 106656.10948942904,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Span Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.005939,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3675797.4628796284,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3640489.917867185,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": -2501,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Dropped Spans %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 47.36728619505037,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 53.61234183555521,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 21.839453125,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.18359375,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "spans_produced_rate",
            "value": 1666.6045856458513,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Span Throughput"
          },
          {
            "name": "spans_received_rate",
            "value": 43348.38527264859,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Span Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002235,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 5548983.274215941,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5503360.024557238,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Network Utilization"
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
        "date": 1776109020993,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_signals_percentage",
            "value": -5466,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Dropped Metrics %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 52.799800507194625,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 54.227625877091526,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.205598958333333,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.93359375,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "metrics_produced_rate",
            "value": 1666.603696823655,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Metric Throughput"
          },
          {
            "name": "metrics_received_rate",
            "value": 92763.16176520463,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Metric Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002267,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3827247.7089507068,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3786063.1624051384,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": -1125,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Dropped Metrics %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 16.28938456047128,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 18.650877030162412,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.501302083333332,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.7890625,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "metrics_produced_rate",
            "value": 1666.453638343232,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Metric Throughput"
          },
          {
            "name": "metrics_received_rate",
            "value": 20414.05706970459,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Metric Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00767,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 818042.2739397618,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 778990.9484583992,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Dropped Metrics %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 17.223194872242182,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 17.98910356562766,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 21.500520833333333,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 22.375,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "metrics_produced_rate",
            "value": 104995.07573094821,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Metric Throughput"
          },
          {
            "name": "metrics_received_rate",
            "value": 104995.07573094821,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Metric Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002814,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 946552.1017455885,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 923562.4836217017,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Dropped Spans %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 30.791989936071452,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 32.143948379167306,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 49.367838541666664,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 52.80078125,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "spans_produced_rate",
            "value": 104995.0722312766,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Span Throughput"
          },
          {
            "name": "spans_received_rate",
            "value": 104995.0722312766,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Span Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002816,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 684675.5094031048,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3624268.588050105,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1240,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 11.369904558977026,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 11.975883627359952,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.130598958333334,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.55859375,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1666.6011970163104,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 22332.45604001856,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002357,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 496972.78850296413,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 475702.258291404,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.5714285373687744,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 14.44331803452607,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 15.558383682299311,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 31.994401041666666,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 34.18359375,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104993.46765642065,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106643.36500530726,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003733,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 144772.32480919582,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 537314.276556195,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Dropped Metrics %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 17.193817857144865,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 18.485881267302368,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 30.695052083333334,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 32.06640625,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "metrics_produced_rate",
            "value": 104995.97515428576,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Metric Throughput"
          },
          {
            "name": "metrics_received_rate",
            "value": 104995.97515428576,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Metric Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.0023,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 226309.02849670133,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 915599.0109705587,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": -914.9999389648438,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Dropped Spans %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 21.40252041599026,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 22.579025823411165,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 27.295572916666668,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 29.046875,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "spans_produced_rate",
            "value": 1666.603085758945,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Span Throughput"
          },
          {
            "name": "spans_received_rate",
            "value": 16916.021320453292,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Span Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002289,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2056862.9481681562,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2036592.9251947952,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": -2636,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Dropped Spans %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 50.390847139581176,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 54.37617890582682,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 22.236328125,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 24.00390625,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "spans_produced_rate",
            "value": 1666.604057907558,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Span Throughput"
          },
          {
            "name": "spans_received_rate",
            "value": 45598.287024350786,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Span Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002254,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 5895214.0222208705,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5860593.525571546,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 17.07030735550444,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 18.755599505562422,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 23.074479166666666,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.953125,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104982.92627675652,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 104982.92627675652,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.009758,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 575166.1415075914,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 546930.2152504136,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Dropped Spans %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 27.783928106158605,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 29.10700707474623,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 27.8703125,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 28.81640625,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "spans_produced_rate",
            "value": 104995.8194164569,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Span Throughput"
          },
          {
            "name": "spans_received_rate",
            "value": 104995.8194164569,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Span Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002389,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3662195.191993929,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3629007.4255820545,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -38.46666717529297,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 47.41148633846253,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 49.65298242898057,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.217838541666666,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.9765625,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 74997.07511407055,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 103845.9500079497,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00234,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2483729.623997414,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2444273.858803066,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Network Utilization"
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
        "date": 1776133890084,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_signals_percentage",
            "value": -0.8571428060531616,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Dropped Spans %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 31.69566484271068,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 33.37658325610133,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 50.02330729166667,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 57.30078125,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "spans_produced_rate",
            "value": 104987.03934999224,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Span Throughput"
          },
          {
            "name": "spans_received_rate",
            "value": 105886.92825870647,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Span Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.007407,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 691634.4333236179,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3658790.242266054,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": -930,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Dropped Spans %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 21.37474735843904,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 22.657753696106507,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 26.196223958333334,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 30.02734375,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "spans_produced_rate",
            "value": 1666.5916144909609,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Span Throughput"
          },
          {
            "name": "spans_received_rate",
            "value": 17165.893629256894,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Span Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002702,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2050558.8622330837,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2048458.7494628937,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Dropped Metrics %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 17.236762125584708,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 18.226147186147188,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 22.482421875,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 22.99609375,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "metrics_produced_rate",
            "value": 106664.11917195377,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Metric Throughput"
          },
          {
            "name": "metrics_received_rate",
            "value": 106664.11917195377,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Metric Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001433,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 942333.5300391711,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 909665.3376069933,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": -0.4687500298023224,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Dropped Spans %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 28.01823921502123,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 30.3357516492045,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 28.253255208333332,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 29.33984375,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "spans_produced_rate",
            "value": 106663.13433920113,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Span Throughput"
          },
          {
            "name": "spans_received_rate",
            "value": 107163.11778141613,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Span Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001987,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3674564.2602605848,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3623583.1046481724,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.5625,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 16.910059849525354,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 17.319894818252127,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 22.859765625,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.51953125,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106656.56806728015,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108323.07694333141,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.005681,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 566361.4786527804,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 535079.0991045077,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": -1136,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Dropped Metrics %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 16.44657776192101,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 18.763534449202023,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.201953125,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.65234375,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "metrics_produced_rate",
            "value": 1666.4633025949734,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Metric Throughput"
          },
          {
            "name": "metrics_received_rate",
            "value": 20597.48642007387,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Metric Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.007322,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 820308.9964584094,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 780487.2719090439,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": -5376,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Dropped Metrics %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 52.91234453317693,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 54.79567149160152,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.988541666666666,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.49609375,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "metrics_produced_rate",
            "value": 1666.5423981551776,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Metric Throughput"
          },
          {
            "name": "metrics_received_rate",
            "value": 91259.86172297753,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Metric Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004474,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3782250.142803105,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3741657.8641198543,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": -2716,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Dropped Spans %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 52.967070595701635,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 55.60176949941793,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 21.474348958333334,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.1171875,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "spans_produced_rate",
            "value": 1666.6131128319744,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Span Throughput"
          },
          {
            "name": "spans_received_rate",
            "value": 46931.8252573484,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Span Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001928,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 6082713.061023885,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 6047670.314555321,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Dropped Metrics %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 17.040574588343453,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 17.6689786075457,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 30.875390625,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 32.23828125,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "metrics_produced_rate",
            "value": 106662.92457572947,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Metric Throughput"
          },
          {
            "name": "metrics_received_rate",
            "value": 106662.92457572947,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Metric Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002105,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 226039.86494165836,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 905517.5025223271,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 14.187026669281924,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 14.606202414113278,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 32.044140625,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 33.875,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106660.11773543771,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106660.11773543771,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003684,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 143564.39040067393,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 533083.1360526463,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -25.098041534423828,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 48.7666114425308,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 49.95030875183781,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.370833333333334,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.18359375,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 84996.64829883541,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106329.14042089606,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002366,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2432319.7863588426,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2392921.336557963,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1250,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 11.523387542631232,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 12.421107330652644,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.991015625,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.7109375,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1666.57156098292,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 22498.716073269417,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003424,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 501987.14332821727,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 483805.47545237205,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Network Utilization"
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
        "date": 1776361902880,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_signals_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Dropped Metrics %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 17.3927618065997,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 18.505802687140115,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 22.322005208333334,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.3515625,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "metrics_produced_rate",
            "value": 104996.11164399878,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Metric Throughput"
          },
          {
            "name": "metrics_received_rate",
            "value": 104996.11164399878,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Metric Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002222,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 949929.9143852869,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 922411.1561030262,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Dropped Spans %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 30.923613664592807,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 32.039352750809066,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 49.83997395833333,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 54.5,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "spans_produced_rate",
            "value": 104987.03934999224,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Span Throughput"
          },
          {
            "name": "spans_received_rate",
            "value": 104987.03934999224,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Span Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.007407,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 685163.393806449,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3613619.108348842,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.5555555820465088,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 23.221772741859535,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 24.28131406044678,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 53.71432291666667,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 59.89453125,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104996.12564296377,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 105579.43745209134,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002214,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2557770.5345297656,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2512220.4199636197,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": 99.94737243652344,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Dropped Spans %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 55.468220567712734,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 88.4719164410058,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 98.68098958333333,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 204.88671875,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "spans_produced_rate",
            "value": 63330.88981650124,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Span Throughput"
          },
          {
            "name": "spans_received_rate",
            "value": 33.33204727184276,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Span Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002315,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 4560239.900462574,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 7835507.039407636,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 100,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 29.375067219444595,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 30.413588057854433,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 50.465885416666666,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 68.62890625,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104985.5504887344,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.008258,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2286989.2032336625,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2498468.3318706523,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Dropped Metrics %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 16.978301471306505,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 17.57775612773525,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 30.564322916666665,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 31.42578125,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "metrics_produced_rate",
            "value": 106662.79836251272,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Metric Throughput"
          },
          {
            "name": "metrics_received_rate",
            "value": 106662.79836251272,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Metric Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002176,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 227228.62986068518,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 909878.4069481394,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": -1.328125,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Dropped Spans %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 81.88119023471502,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 92.20659401808487,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 85.32682291666667,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 109.91015625,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "spans_produced_rate",
            "value": 106659.51870125503,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Span Throughput"
          },
          {
            "name": "spans_received_rate",
            "value": 108076.09043400608,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Span Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004021,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 13986396.964091588,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 14044908.106107282,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 14.086179793006979,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 14.567317676143388,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 32.400390625,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 34.26953125,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106661.23760967233,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106661.23760967233,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003054,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 142370.50314783378,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 529405.4310997047,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": -1.5873017311096191,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Dropped Spans %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 28.073546415878276,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 30.381981926314978,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 27.63828125,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 29.4609375,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "spans_produced_rate",
            "value": 104995.99790254662,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Span Throughput"
          },
          {
            "name": "spans_received_rate",
            "value": 106662.60104385688,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Span Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002287,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3673226.349221553,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3638331.203861168,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": 100,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Dropped Metrics %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 43.484342458716384,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 45.047298570876784,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 45.34908854166667,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 69.08984375,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "metrics_produced_rate",
            "value": 104995.96990468849,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Metric Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002303,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 4308088.503392677,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 4441710.1590268975,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.0158729553222656,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 17.060258025322202,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 18.39489137997681,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 23.122526041666667,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 24.5078125,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104983.43186472787,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106049.93022017909,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.009469,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 574614.3316362924,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 542779.9352471147,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": -1.5873017311096191,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Dropped Metrics %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 35.69587898939204,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 36.759755739352244,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 40.11276041666667,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 44.00390625,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "metrics_produced_rate",
            "value": 104996.09064555829,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Metric Throughput"
          },
          {
            "name": "metrics_received_rate",
            "value": 106662.69525897986,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Metric Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002234,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 4450974.914325446,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 4444212.181041342,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Network Utilization"
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
        "date": 1776392110859,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_signals_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Dropped Spans %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 30.975362070524852,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 32.34699751861042,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 50.07890625,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 54.19140625,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "spans_produced_rate",
            "value": 106653.49140536172,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Span Throughput"
          },
          {
            "name": "spans_received_rate",
            "value": 106653.49140536172,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Span Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.007412,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 688145.4320532721,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3620792.7530545196,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": -1.5873017311096191,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Dropped Metrics %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 35.705286727572826,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 36.614020046260606,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 44.15052083333333,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 48.1484375,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "metrics_produced_rate",
            "value": 104989.13187503206,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Metric Throughput"
          },
          {
            "name": "metrics_received_rate",
            "value": 106655.62603177861,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Metric Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.006211,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 4479279.108944299,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 4450111.580455315,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": -1.5714285373687744,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Dropped Metrics %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 17.346442811172782,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 18.580797782227013,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 30.742578125,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 31.57421875,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "metrics_produced_rate",
            "value": 104996.18163886108,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Metric Throughput"
          },
          {
            "name": "metrics_received_rate",
            "value": 106646.12163604317,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Metric Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002182,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 229169.23607304774,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 911247.1807538302,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": -1.484375,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Dropped Spans %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 85.93382165921926,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 92.57943447106865,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 93.221484375,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 107.11328125,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "spans_produced_rate",
            "value": 106644.78493421126,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Span Throughput"
          },
          {
            "name": "spans_received_rate",
            "value": 108227.79346057845,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Span Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.012311,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 14030099.230276916,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 14040141.24366784,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Dropped Spans %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 28.001959675987326,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 29.53506762500966,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 27.106119791666668,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 29.33984375,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "spans_produced_rate",
            "value": 106662.73614483973,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Span Throughput"
          },
          {
            "name": "spans_received_rate",
            "value": 106662.73614483973,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Span Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002211,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3702663.5007658633,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3652324.3499031034,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": 99.94737243652344,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Dropped Spans %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 54.54561502799299,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 88.19203958558836,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 95.88255208333334,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 148.6171875,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "spans_produced_rate",
            "value": 63330.92359169067,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Span Throughput"
          },
          {
            "name": "spans_received_rate",
            "value": 33.33206504825825,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Span Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002283,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2802685.6961033302,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 7819497.544542249,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": 100,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Dropped Metrics %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 43.07243859511413,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 44.385213445119604,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 51.141796875,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 66.33984375,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "metrics_produced_rate",
            "value": 106662.80014016158,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Metric Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002175,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 4403338.177839423,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 4365384.413075785,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Dropped Metrics %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 17.42149106488677,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 18.437858078434395,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 21.5625,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 22.72265625,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "metrics_produced_rate",
            "value": 104996.17813911574,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Metric Throughput"
          },
          {
            "name": "metrics_received_rate",
            "value": 104996.17813911574,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Metric Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002184,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 957004.6575321144,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 930460.687132258,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.4285714626312256,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 23.299989059919998,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 24.676963043142106,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 51.76002604166667,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 55.44140625,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104996.14489154673,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106496.08981856883,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002203,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2538409.353987479,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2493141.005620243,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.5873017311096191,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 16.944470963001567,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 17.884383985159992,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 22.808723958333335,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.8828125,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104985.88114875018,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106652.32370666685,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.008069,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 568842.0676756845,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 537407.2051555002,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 100,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 29.082522963850554,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 31.74718491615795,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 54.75533854166667,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 91.20703125,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106660.78432441117,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003309,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2300434.0744067184,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2430788.820982362,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.5714285373687744,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 14.34961902226446,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 16.059421912136777,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 31.480729166666666,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 33.98046875,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104994.46329196908,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106644.37628655716,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003164,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 144521.31534608014,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 535348.6088282493,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Network Utilization"
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
        "date": 1776447564058,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 98.46875,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 30.823559840859243,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 41.53100371747212,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 64.94596354166667,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 109.0625,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106662.23662843871,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1633.2654983729676,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002492,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2338708.360590786,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2465973.9562301505,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": 98.078125,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Dropped Metrics %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 45.42840188271642,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 46.207890776510745,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 51.11770833333333,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 63.33984375,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "metrics_produced_rate",
            "value": 106653.52517480637,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Metric Throughput"
          },
          {
            "name": "metrics_received_rate",
            "value": 2049.74743695331,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Metric Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.007393,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 4298967.956112103,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 4370434.1465190295,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": 2,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Dropped Spans %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 30.46623768327018,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 35.94213921901528,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 45.6046875,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 48.33984375,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "spans_produced_rate",
            "value": 79997.07210716089,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Span Throughput"
          },
          {
            "name": "spans_received_rate",
            "value": 78397.13066501767,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Span Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002196,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2972640.948921523,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3222630.052230625,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": 1.171875,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Dropped Metrics %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 36.59718477136057,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 39.23591034694504,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 47.45104166666667,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 52.59765625,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "metrics_produced_rate",
            "value": 106662.81080605602,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Metric Throughput"
          },
          {
            "name": "metrics_received_rate",
            "value": 105412.85599192255,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Metric Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002169,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 4465723.409004649,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 4374183.432282045,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.5625,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 16.86232831589654,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 17.110196017904,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.888671875,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 21.55078125,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106662.06064334788,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 104995.46594579557,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002591,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 778543.4036767004,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 747991.8956801094,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 23.599865708486284,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 25.29834584139265,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 57.106380208333334,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 60.05859375,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104987.27029347692,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 104987.27029347692,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.007275,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2543290.18679381,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2464634.130166173,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.5625,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 15.78675388386355,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 16.428051305825996,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 29.735546875,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 31.83984375,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106657.45501780164,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 104990.93228314849,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.005182,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 561422.0272505869,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 743045.8894610255,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": 1.5625,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Dropped Metrics %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 19.547014661883644,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 20.06178021808058,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 22.448567708333332,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 22.7734375,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "metrics_produced_rate",
            "value": 106662.80725075764,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Metric Throughput"
          },
          {
            "name": "metrics_received_rate",
            "value": 104996.20088746455,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Metric Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002171,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 1301843.8838828686,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1268975.529589052,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": 1.5625,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Dropped Metrics %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 20.10052673440928,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 20.609803375135467,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 33.585546875,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 36.40625,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "metrics_produced_rate",
            "value": 106662.81258370531,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Metric Throughput"
          },
          {
            "name": "metrics_received_rate",
            "value": 104996.20613708491,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Metric Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002168,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 930471.9626239878,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1268660.7917063846,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Network Utilization"
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
        "date": 1776481889221,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_signals_percentage",
            "value": 98.046875,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Dropped Metrics %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 45.26805835014028,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 46.501984544049456,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 46.981510416666666,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 63.11328125,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "metrics_produced_rate",
            "value": 106662.71125779086,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Metric Throughput"
          },
          {
            "name": "metrics_received_rate",
            "value": 2083.2560792537274,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Metric Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002225,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 4365469.90205859,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 4363299.004554307,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": 1.09375,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Dropped Metrics %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 36.45881250010507,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 37.67830838092298,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 44.74817708333333,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 49.5625,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "metrics_produced_rate",
            "value": 106661.68201072737,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Metric Throughput"
          },
          {
            "name": "metrics_received_rate",
            "value": 105495.06986373504,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Metric Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002804,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 4454701.604243948,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 4374064.624218233,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": 6.800000190734863,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Dropped Spans %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 8.61935322199289,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 34.88389125551855,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 43.86966145833333,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 48.49609375,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "spans_produced_rate",
            "value": 24996.962035880573,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Span Throughput"
          },
          {
            "name": "spans_received_rate",
            "value": 23297.168617440693,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Span Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.007292,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 886077.3373695516,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 925478.9196855497,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 23.917136180100186,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 26.45330056071895,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 56.2109375,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 62.203125,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106662.78591897231,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106662.78591897231,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002183,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2555550.208297431,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2470531.305509156,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Dropped Metrics %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 19.8274865988466,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 20.970040185471404,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 23.07421875,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 24.34375,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "metrics_produced_rate",
            "value": 106653.59626844396,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Metric Throughput"
          },
          {
            "name": "metrics_received_rate",
            "value": 106653.59626844396,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Metric Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.007353,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 1326341.2528081497,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1278948.79179871,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 98.203125,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 29.735908944502786,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 38.15732673267327,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 68.90572916666666,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 100.76171875,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106662.18507719034,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1916.586138105764,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002521,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2312139.2919068807,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2504447.9054737333,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Dropped Metrics %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 20.19276848212631,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 22.59739520494973,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 33.821875,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 37.15625,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "metrics_produced_rate",
            "value": 104995.95765563025,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Metric Throughput"
          },
          {
            "name": "metrics_received_rate",
            "value": 104995.95765563025,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Metric Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00231,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 943905.4825404428,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1276127.121309783,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 15.879715968620195,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 17.642674893945237,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 33.40963541666667,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 37.16796875,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106661.0438519716,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106661.0438519716,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003163,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 568056.8215436074,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 752696.5300475811,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 18.283378410144703,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 19.42852557673019,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 23.888020833333332,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 25.32421875,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104994.23581645369,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 104994.23581645369,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003294,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 787728.9618868337,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 748564.7890983914,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Network Utilization"
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
        "date": 1776533269228,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.5625,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 17.056263833299703,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 18.142802350757812,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.243098958333334,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.73828125,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106650.65307110804,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 104984.23661687197,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.009009,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 783012.5255948991,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 755716.2256235868,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": 3.309523582458496,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Dropped Spans %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 21.29862800297607,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 31.413125681358046,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 25.343098958333332,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 26.77734375,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "spans_produced_rate",
            "value": 69997.49992262777,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Span Throughput"
          },
          {
            "name": "spans_received_rate",
            "value": 67680.915996617,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Span Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002143,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2818634.4785134722,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2778160.2473046687,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 98.3125,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 30.393987784926797,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 33.66764869029276,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 58.9484375,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 89.6171875,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106653.72423723048,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1799.7815965032644,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.007281,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2361469.603281135,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2492966.2862599543,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": 0.4687500298023224,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Dropped Metrics %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 36.4135981229073,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 37.228177385651264,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 43.81653645833333,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 48.93359375,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "metrics_produced_rate",
            "value": 106662.71303543683,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Metric Throughput"
          },
          {
            "name": "metrics_received_rate",
            "value": 106162.73156808321,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Metric Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002224,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 4433107.639337076,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 4396352.373892912,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": 2.0199999809265137,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Dropped Spans %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 31.894435911669543,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 35.98517302779283,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 45.594401041666664,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 49.19140625,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "spans_produced_rate",
            "value": 83330.45426613843,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Span Throughput"
          },
          {
            "name": "spans_received_rate",
            "value": 81647.17908996243,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Span Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002073,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3091221.0283953077,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3364267.570656189,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": 98.046875,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Dropped Metrics %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 45.26390910290053,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 46.928252805129375,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 50.82955729166667,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 65.83203125,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "metrics_produced_rate",
            "value": 106662.74503307429,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Metric Throughput"
          },
          {
            "name": "metrics_received_rate",
            "value": 2083.256738927232,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Metric Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002206,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 4402876.690208482,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 4374426.287195806,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": 1.5625,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Dropped Metrics %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 20.085342575601143,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 21.260101460415065,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 34.333203125,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 37.27734375,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "metrics_produced_rate",
            "value": 106662.67926017365,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Metric Throughput"
          },
          {
            "name": "metrics_received_rate",
            "value": 104996.07489673344,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Metric Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002243,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 924859.7577234418,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1276108.2210240776,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.5625,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 23.81696813556246,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 24.740373745173745,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 53.48776041666667,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 61.55859375,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106662.752143663,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 104996.14664141827,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002202,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2530837.654696064,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2483286.9845052385,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": -1.5873017311096191,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Dropped Metrics %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 19.675210381864446,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 20.40186910103667,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 22.988411458333335,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.45703125,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "metrics_produced_rate",
            "value": 104996.02765028723,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Metric Throughput"
          },
          {
            "name": "metrics_received_rate",
            "value": 106662.63126378384,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Metric Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00227,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 1301730.8226519697,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1268429.2467677416,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.5625,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 15.813772169415355,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 16.656831683168317,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 34.188541666666666,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 36.87109375,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106648.42623083366,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 104982.04457097688,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.010262,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 563076.7865661841,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 753505.1677690858,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Network Utilization"
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
        "date": 1776564726907,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_signals_percentage",
            "value": 1.5625,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Dropped Metrics %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 19.922389772053975,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 20.35489833641405,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 34.11861979166667,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 36.91015625,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "metrics_produced_rate",
            "value": 106662.81436135466,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Metric Throughput"
          },
          {
            "name": "metrics_received_rate",
            "value": 104996.20788695848,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Metric Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002167,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 926418.8391571554,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1277375.6161889047,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 23.905818754459247,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 25.616280744770513,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 54.243359375,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 60.4609375,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106662.63304142715,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106662.63304142715,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002269,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2539200.6392824124,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2528072.8642934286,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.5625,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 17.091533125503716,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 18.62706453851492,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.745182291666666,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 21.86328125,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106660.24927500194,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 104993.68288008004,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00361,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 783743.0137025558,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 759485.8817444582,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.5625,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 15.913204190358979,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 16.92309308613364,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 31.098177083333333,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 32.8046875,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106657.61499040782,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 104991.0897561827,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.005092,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 565833.5061995984,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 755733.956526231,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 97.921875,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 30.20509086307429,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 31.77494613727301,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 61.893619791666666,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 94.62109375,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106662.70236956194,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 2216.584283617459,
            "unit": "logs/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00223,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2362518.431411273,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2483156.977237757,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Logs-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": 98.046875,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Dropped Metrics %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 45.297202960531344,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 46.171886807001314,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 46.897526041666666,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 64.91796875,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "metrics_produced_rate",
            "value": 106662.72370131384,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Metric Throughput"
          },
          {
            "name": "metrics_received_rate",
            "value": 2083.256322291286,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Metric Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002218,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 4405789.976427146,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 4369515.828944204,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": 2.140000104904175,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Dropped Spans %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 31.070045487043767,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 36.08275178184071,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 44.88658854166667,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 47.046875,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - RAM (MiB)"
          },
          {
            "name": "spans_produced_rate",
            "value": 83330.3528843785,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Span Throughput"
          },
          {
            "name": "spans_received_rate",
            "value": 81547.0833326528,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Span Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002146,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3086251.3888340862,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3358299.935025127,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-BATCH-OTAP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": -0.9069767594337463,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Dropped Spans %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 55.26040297335172,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 87.92728961833242,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 71.84388020833333,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 103.65625,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "spans_produced_rate",
            "value": 71664.20022377562,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Span Throughput"
          },
          {
            "name": "spans_received_rate",
            "value": 72314.1778537122,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Span Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002065,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 9558686.283639345,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 9651573.501201907,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": 2.2200000286102295,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Dropped Spans %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 26.927110302198237,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 29.599505562422745,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 25.9,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 27.3671875,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "spans_produced_rate",
            "value": 83323.29843075898,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Span Throughput"
          },
          {
            "name": "spans_received_rate",
            "value": 81473.52120559613,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Span Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.007226,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3409868.0109493495,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3367831.2244362733,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": 100,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Dropped Spans %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 39.005815652897255,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 87.92005560274924,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 89.6828125,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 190.265625,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - RAM (MiB)"
          },
          {
            "name": "spans_produced_rate",
            "value": 44998.17682386902,
            "unit": "spans/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Span Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002431,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2114344.3866720037,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5605403.174339879,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Traces-OTLP-BATCH-OTLP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": 1.5625,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Dropped Metrics %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 19.535984908336403,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 19.900512342803914,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 22.508203125,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.6015625,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "metrics_produced_rate",
            "value": 106653.69046765976,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Metric Throughput"
          },
          {
            "name": "metrics_received_rate",
            "value": 104987.22655410258,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Metric Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.0073,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 1304897.542495592,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1270594.549682426,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTAP-OTAP - Network Utilization"
          },
          {
            "name": "dropped_signals_percentage",
            "value": -1.5625,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Dropped Metrics %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 36.51521943111641,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 38.09019421202119,
            "unit": "%",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 44.3046875,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 52.45703125,
            "unit": "MiB",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "metrics_produced_rate",
            "value": 106662.88902268045,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Metric Throughput"
          },
          {
            "name": "metrics_received_rate",
            "value": 108329.49666365983,
            "unit": "metrics/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Metric Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002125,
            "unit": "seconds",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 4418735.358627408,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 4441213.251174179,
            "unit": "bytes/sec",
            "extra": "Nightly - Batch Processor/Metrics-OTLP-OTLP - Network Utilization"
          }
        ]
      }
    ]
  }
}