window.BENCHMARK_DATA = {
  "lastUpdate": 1774144775788,
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
      }
    ]
  }
}