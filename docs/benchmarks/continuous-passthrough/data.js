window.BENCHMARK_DATA = {
  "lastUpdate": 1768724562586,
  "repoUrl": "https://github.com/open-telemetry/otel-arrow",
  "entries": {
    "Benchmark": [
      {
        "commit": {
          "author": {
            "email": "cijo.thomas@gmail.com",
            "name": "Cijo Thomas",
            "username": "cijothomas"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "b793a1e733d3f1c1a626430c0d93e00e9e6d98e2",
          "message": "PerfTest - add passthrough scenario (#1810)\n\nAll existing tests add a dummy processor in the middle to force\nconversion into internal format. But there are real scenarios where\nengine can act as \"pass-through\" without having to do this conversion.\nThis adds a perf-test to continuously measure the throughput when\noperating as pass-through.\nModelled after saturation tests - where we put as much load as required\nto hit 100% CPU in engine. Local run shows it can do twice (minimum) the\nthroughput!",
          "timestamp": "2026-01-16T22:14:05Z",
          "tree_id": "1cf5cc0d17331750aa5a89bae24befe3b9d85c4a",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/b793a1e733d3f1c1a626430c0d93e00e9e6d98e2"
        },
        "date": 1768603839941,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 97.88627355647486,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.35741167310366,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 44.70872395833333,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 47.7265625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 565936.8956417819,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 565936.8956417819,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001149,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 10151891.240092983,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 10137779.0385659,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "cijo.thomas@gmail.com",
            "name": "Cijo Thomas",
            "username": "cijothomas"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "c68e70eda406b6341cbd0ae73cf4521a56639d47",
          "message": "Update batch size variation perf tests (#1809)\n\nModified to use 10, 100, 512, 1024, 4096, 8192 as sizes.\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2026-01-16T23:41:49Z",
          "tree_id": "2ebd0b963e9f0a0c3a4e59c7f3429710cd874ea8",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/c68e70eda406b6341cbd0ae73cf4521a56639d47"
        },
        "date": 1768609059170,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.16226525157725,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.8441080331295,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 45.4703125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 47.8359375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 570846.5889158417,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 570846.5889158417,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000821,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 10353526.431908403,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 10311706.708369318,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "jmacd@users.noreply.github.com",
            "name": "Joshua MacDonald",
            "username": "jmacd"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "8470d2d442782c9e6dadf2b9379160f88ccc2c39",
          "message": "Split opentelemetry_client into otel_sdk, tracing_init, and ITS parts (#1808)\n\nPart of https://github.com/open-telemetry/otel-arrow/pull/1771.\n\nPart of https://github.com/open-telemetry/otel-arrow/issues/1736.\n\nThis is a non-functional refactoring of `opentelemetry_client.rs` into\nother places. This will make it clearer what changes in #1771 and what\nis just moving around.\n\nMoves runtime elements into the InternalTelemetrySystem, simplifies\nsetup for the controller where logs/metrics were separated.\n\nMoves OTel-SDK specific pieces into `otel_sdk` module, separates the\nTokio `tracing` setup.\n\n---------\n\nCo-authored-by: Utkarsh Umesan Pillai <66651184+utpilla@users.noreply.github.com>",
          "timestamp": "2026-01-17T02:49:23Z",
          "tree_id": "0d830a7035fae4fc9093f5ad8a0572cb4a6bc8c0",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/8470d2d442782c9e6dadf2b9379160f88ccc2c39"
        },
        "date": 1768621823511,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 95.70227227680343,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.67590817445002,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 45.09010416666667,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 47.9921875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 560083.2358186145,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 560083.2358186145,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002053,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 10126874.629998853,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 10083547.927030776,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "l.querel@f5.com",
            "name": "Laurent Quérel",
            "username": "lquerel"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "d8a0d6d381f1a9f2968c182c88920cb4ded93cc0",
          "message": "Create entity and expose entity keys via thread locals and task locals (#1785)\n\nThe engine now creates the following entities:\n\n* **Pipeline** -> Stored in a thread local associated with the pipeline\nthread.\n* **Node** -> Stored in the task local of the node.\n* **Channel**\n  * **Sender entity** stored in the task local of the sender node.\n  * **Receiver entity** stored in the task local of the receiver node.\n\nAn entity cleanup mechanism is in place. A unit test has been added to\nvalidate this cleanup process.\n\nThe final goal is to be able to use these entities directly when\nreporting metric sets and events. This allows us to report the\nattributes of all our entities using a simple numerical ID.\n\nCloses https://github.com/open-telemetry/otel-arrow/issues/1791",
          "timestamp": "2026-01-18T07:23:23Z",
          "tree_id": "0c4a094815fe796e1d1add0c2bcef4a588b7a0f7",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/d8a0d6d381f1a9f2968c182c88920cb4ded93cc0"
        },
        "date": 1768724562085,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.3163589537143707,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 94.53002363824076,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 95.07931411692307,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 45.49752604166667,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 47.76171875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 558336.1490813495,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 556569.8026016478,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001818,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 10152645.282562107,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 10100967.296759335,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      }
    ]
  }
}