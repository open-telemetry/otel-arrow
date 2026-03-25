window.BENCHMARK_DATA = {
  "lastUpdate": 1774460519801,
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
      }
    ]
  }
}