window.BENCHMARK_DATA = {
  "lastUpdate": 1769822900868,
  "repoUrl": "https://github.com/open-telemetry/otel-arrow",
  "entries": {
    "Benchmark": [
      {
        "commit": {
          "author": {
            "name": "clhain",
            "username": "clhain",
            "email": "8164192+clhain@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "b06c055168630b8a3f2ee859f2d62428e82a89de",
          "message": "[PipelinePerf] setup verification of manual workflow runs for PRs (#1381)\n\nThese changes allow automated verification of a manually triggered perf\ntest workflow for a given PR number. Because of how github actions work\nfor this weird case, it's a 2 step thing:\n\n- The new workflow file below (pipeline-perf-verify-pr.yaml) inserts a\nplaceholder status into PRs on create/update that will remain \"pending\"\nindefinitely.\n- The original manual workflow now updates that placeholder status with\nthe result of it's own run (for the specified PR number).\n\nSo the flow is:\n- New PR created, new job inserts the pending status result\n- Maintainer triggers the manual run with target PR number\n- Manual job runs and sets the previously pending result to pass/fail\n\nNew commits will clear the result back to pending.",
          "timestamp": "2025-11-04T22:41:54Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/b06c055168630b8a3f2ee859f2d62428e82a89de"
        },
        "date": 1762338006198,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 43.74455616712649,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 45.225771815726524,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 29.06796875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 29.37890625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 13006764.50307346,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 12484616.006516013,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 999000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 99.9000015258789,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 0.038101132966065536,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 0.061286491173738,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 28.919921875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 28.99609375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 1000000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 1000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 59090.32577519332,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 4819.591274750636,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
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
          "id": "e9fc2a4d40ea8196454da17a1bfb9dd7c5d2ed8a",
          "message": "[otel-arrow-rust] Replace Snafu with thiserror (#1389)\n\nPart of #1377 #867 \nReplaces snafu with thiserror.\n\nHere's my retrospective on Snafu vs. Thiserror:\n\nSnafu has a little more sugar, maybe too much magic. E.g., options and\nresults both have context extensions, with thiserror you choose\nok_or_else() or map_err() explicitly. While I admire Snafu for its\nautomation, I prefer the more sparing approach taken by thiserror.",
          "timestamp": "2025-11-05T22:16:25Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/e9fc2a4d40ea8196454da17a1bfb9dd7c5d2ed8a"
        },
        "date": 1762424249858,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 43.54232421870965,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 44.516007146063934,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 29.467578125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 29.7265625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 12976749.397470247,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 12462574.988432076,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 999000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 99.9000015258789,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 0.050346933435021134,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 0.07709810526315788,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 28.734765625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 28.93359375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 1000000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 1000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 58819.53739888202,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 4796.581158958952,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
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
          "id": "dbd017c4dbf2305c619c31da35ce0722ff46b369",
          "message": "[query-engine] Add special handling for invalid equality expressions in KQL comparisons (#1400)\n\n## Changes\n\n* Add a rule in KQL pest to allow `where [scalar] = [scalar]` so that a\nmore intentional error message can be generated hinting that `==` is the\nmost likely solution",
          "timestamp": "2025-11-06T23:37:57Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/dbd017c4dbf2305c619c31da35ce0722ff46b369"
        },
        "date": 1762514342060,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 43.74543759482295,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 45.597071588915625,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 25.613671875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 25.9140625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 12980112.233253092,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 12457200.346265713,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 999000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 99.9000015258789,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 0.06313675222598737,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 0.08425040892193308,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 28.856640625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 29.11328125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 1000000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 1000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 59122.76821973984,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 4822.218212316206,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
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
          "id": "183c1a70d3b82a538e6573e9150f61f8b768106f",
          "message": "Update Go collector dependencies v0.139.0 and v1.45.0 (#1401)",
          "timestamp": "2025-11-08T00:05:20Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/183c1a70d3b82a538e6573e9150f61f8b768106f"
        },
        "date": 1762596965207,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 999000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 99.9000015258789,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 0.05284188888058218,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 0.09807419534883721,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 23.9703125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 25.78125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 1000000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 1000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 59111.017750108054,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 4821.271024490977,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 43.452910766389394,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 44.06461682784396,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 29.11328125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 29.515625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 13024015.64826715,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 12509892.36823788,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
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
          "id": "183c1a70d3b82a538e6573e9150f61f8b768106f",
          "message": "Update Go collector dependencies v0.139.0 and v1.45.0 (#1401)",
          "timestamp": "2025-11-08T00:05:20Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/183c1a70d3b82a538e6573e9150f61f8b768106f"
        },
        "date": 1762683369030,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 43.23294908600279,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 44.40442050425367,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 27.373046875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 27.65234375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 12937984.938547574,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 12415165.691697847,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 999000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 99.9000015258789,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 0.050282678209843515,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 0.07019596932847959,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 27.346484375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 27.5625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 1000000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 1000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 59118.25938031198,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 4821.841118982979,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
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
          "id": "183c1a70d3b82a538e6573e9150f61f8b768106f",
          "message": "Update Go collector dependencies v0.139.0 and v1.45.0 (#1401)",
          "timestamp": "2025-11-08T00:05:20Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/183c1a70d3b82a538e6573e9150f61f8b768106f"
        },
        "date": 1762771073670,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 43.31690935544785,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 44.23595704415182,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 29.4421875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 29.76953125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 13056114.38243676,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 12501331.78983486,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 999000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 99.9000015258789,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 0.04427465372904026,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 0.07366559282733035,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 25.55703125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 25.6328125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 1000000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 1000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 59127.854344591484,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 4822.646265302006,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
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
          "id": "ba0400f949aa76100aeed43d4506736a531062be",
          "message": "Use GITHUB_STEP_SUMMARY on release action dry_run (#1412)\n\nInspired by usage in other repositories - nicer to avoid digging through\ndetailed dry run logs to ensure release looks as expected.",
          "timestamp": "2025-11-10T23:40:26Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/ba0400f949aa76100aeed43d4506736a531062be"
        },
        "date": 1762856218155,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 44.44610423124621,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 46.750111306501545,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 25.57734375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 25.87109375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 13135242.941642415,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 12620197.40143602,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 999000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 99.9000015258789,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 0.04953275897182785,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 0.07972412595005429,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 28.928125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 29.1640625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 1000000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 1000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 59063.42423282591,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 4817.468882716184,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "otelbot[bot]",
            "username": "otelbot[bot]",
            "email": "197425009+otelbot[bot]@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "430a4835157fdbe9cbf98b75970ff971476a39af",
          "message": "chore(release) Prepare Release v0.45.0 (#1415)\n\n## Release v0.45.0\n\nThis PR prepares the repository for release v0.45.0.\n\n### Changes included:\n- Updated CHANGELOG.md with release notes\n- Updated collector/otelarrowcol-build.yaml version to v0.45.0\n- Updated collector/cmd/otelarrowcol/main.go version to v0.45.0\n\n### Release Notes:\n- Upgrade to v0.139.0 / v1.45.0 of collector dependencies.\n[#1401](https://github.com/open-telemetry/otel-arrow/pull/1401)\n\n### Checklist:\n- [x] Verify CHANGELOG.md formatting and content\n- [x] Verify collector version update in\ncollector/otelarrowcol-build.yaml\n- [x] Verify collector main.go version update in\ncollector/cmd/otelarrowcol/main.go\n- [x] Confirm all tests pass\n- [x] Ready to merge and tag release\n\nAfter merging this PR, run the **Push Release** workflow to create git\ntags and publish the GitHub release.\n\nCo-authored-by: otelbot <197425009+otelbot@users.noreply.github.com>",
          "timestamp": "2025-11-11T16:33:50Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/430a4835157fdbe9cbf98b75970ff971476a39af"
        },
        "date": 1762942642687,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 44.622829507611144,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 46.56869106330289,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 29.1109375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 29.375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 13331845.629141565,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 12727244.70189437,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 999000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 99.9000015258789,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 0.05927566201190807,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 0.07671064225876512,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 27.21875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 27.35546875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 1000000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 1000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 59112.96943154108,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 4821.373560219042,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
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
          "id": "1477cd3640ddf45c083cf9170288010ce185bb45",
          "message": "Support expanded and packed encoding for repeated primitive fields when decoding protobuf (#1424)\n\ncloses #1399 \n\nAccording to the [protobuf encoding\ndocs](https://protobuf.dev/programming-guides/encoding/), there's two\ncases decoders need to handle:\n\n> Protocol buffer parsers must be able to parse repeated fields that\nwere compiled as packed as if they were not packed, and vice versa.\n\nAlso, if the encoding is \"packed\", we need to handle if there are more\nthan one segment of the buffer containing packed values\n> Note that although there’s usually no reason to encode more than one\nkey-value pair for a packed repeated field, parsers must be prepared to\naccept multiple key-value pairs. In this case, the payloads should be\nconcatenated. Each pair must contain a whole number of elements. The\nfollowing is a valid encoding of the same message above that parsers\nmust accept\n\nThis PR brings our decoding logic in line with the recommendations for\nthe few fields we have that could have this encoding. Before this\nchange, we were assuming the encoding was always packed.\n\nThis change implements a new iterator for these types called\n`RepeatedPrimitiveIter` which encapsulates the logic of determining how\nto iterate the buffer and produce values correctly for both types of\nencodings. Internally, it use a combination of the\n`RepeatedFieldProtoBytesParser` and the packed field iterators which\nwere already implemented before this change.\n\n---------\n\nCo-authored-by: Joshua MacDonald <jmacd@users.noreply.github.com>\nCo-authored-by: Utkarsh Umesan Pillai <66651184+utpilla@users.noreply.github.com>",
          "timestamp": "2025-11-12T21:44:53Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/1477cd3640ddf45c083cf9170288010ce185bb45"
        },
        "date": 1763029206683,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 999000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 99.9000015258789,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 0.05410845754342978,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 0.07470818089826418,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 29.017578125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 29.15234375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 1000000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 1000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 59118.98396137238,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 4821.900567964224,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 44.69616344419765,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 46.9650447761194,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 29.034765625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 29.30078125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 13158030.49434839,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 12630985.647794561,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
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
          "id": "cb2f946eed228de75fd07cf4095888bb945b0b21",
          "message": "Rename otap_batch_processor to batch_processor (#1432)\n\nThe additional \"otap_\" in this component name is confusing/unnecessary.\nThe URN is still \"urn:otap:processor:batch\".\n\nThe otap_batch_processor/ subdir had only a metrics struct, moved it\ninto the main code to simplify.",
          "timestamp": "2025-11-14T00:35:41Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/cb2f946eed228de75fd07cf4095888bb945b0b21"
        },
        "date": 1763115449247,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 999000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 99.9000015258789,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 0.04010898133642842,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 0.08135149674620391,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 25.534375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 25.6640625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 1000000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 1000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 59129.96919649616,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 4822.824546705401,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 44.97629124836777,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 47.14687158025074,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 23.286328125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.58984375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 13243259.67773273,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 12673751.372251201,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
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
          "id": "e771ca91cbfb6a0158ef31772b6cc9aba68fa044",
          "message": "[query-engine] Fix strcat appending \"null\" bug (#1434)\n\n## Changes\n\n* Fixes recordset engine so that when executing `strcat` it appends\nempty string for `null` instead of \"null\"",
          "timestamp": "2025-11-14T21:38:04Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/e771ca91cbfb6a0158ef31772b6cc9aba68fa044"
        },
        "date": 1763201923140,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 999000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 99.9000015258789,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 0.05674699635959911,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 0.07867576128183296,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 23.611328125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.66796875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 1000000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 1000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 59104.30033592463,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 4805.34873753784,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 14000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.5833333134651184,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 44.56688512505507,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 46.81547476511106,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 29.359765625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 29.50390625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 2386000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 13193551.208524141,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 12669269.557917425,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
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
          "id": "e771ca91cbfb6a0158ef31772b6cc9aba68fa044",
          "message": "[query-engine] Fix strcat appending \"null\" bug (#1434)\n\n## Changes\n\n* Fixes recordset engine so that when executing `strcat` it appends\nempty string for `null` instead of \"null\"",
          "timestamp": "2025-11-14T21:38:04Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/e771ca91cbfb6a0158ef31772b6cc9aba68fa044"
        },
        "date": 1763288206290,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 44.37854036787664,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 46.8200724052579,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 25.63125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 26.015625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 13237534.277193006,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 12718882.119113857,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 999000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 99.9000015258789,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 0.05202000161819554,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 0.07075899305019305,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 23.57265625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.90625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 1000000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 1000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 58826.38373617198,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 4798.483123865454,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
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
          "id": "e771ca91cbfb6a0158ef31772b6cc9aba68fa044",
          "message": "[query-engine] Fix strcat appending \"null\" bug (#1434)\n\n## Changes\n\n* Fixes recordset engine so that when executing `strcat` it appends\nempty string for `null` instead of \"null\"",
          "timestamp": "2025-11-14T21:38:04Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/e771ca91cbfb6a0158ef31772b6cc9aba68fa044"
        },
        "date": 1763374646846,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 999000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 99.9000015258789,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 0.04373494326142531,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 0.0582389593495935,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 23.655859375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.80078125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 1000000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 1000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 58828.95765626036,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 4797.354521877523,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 44.20692297072733,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 46.746399764687666,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 29.373828125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 29.61328125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 13196548.807081494,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 12666976.347801272,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
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
          "id": "43596412cec500363a1226ad3957237d83fbfcf5",
          "message": "Nit fix to readme on running dataflow engine (#1443)",
          "timestamp": "2025-11-18T00:54:02Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/43596412cec500363a1226ad3957237d83fbfcf5"
        },
        "date": 1763462715228,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 999000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 99.9000015258789,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 0.04519118427213611,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 0.05741617387297443,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 23.32890625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.421875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 1000000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 1000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 59121.27166060477,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 4822.122099554957,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 45.06347598904073,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 46.39328332145624,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 25.719921875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 25.91796875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 13144472.12119722,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 12629185.939301156,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
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
          "id": "a2fc3baf6988b2014a28f12006b56757c82ebc37",
          "message": "Fix meeting time (#1450)\n\nLooks like the doc in the repo was outdated.",
          "timestamp": "2025-11-19T00:42:26Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/a2fc3baf6988b2014a28f12006b56757c82ebc37"
        },
        "date": 1763549416210,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 44.70113361917228,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 47.03209848260836,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 27.620703125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 27.76953125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 13187475.897911904,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 12664102.957080709,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 999000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 99.9000015258789,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 0.0461515914237069,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 0.07553238773614121,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 27.4734375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 27.54296875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 1000000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 1000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 59120.076624207795,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 4821.954092161546,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
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
          "id": "a2fc3baf6988b2014a28f12006b56757c82ebc37",
          "message": "Fix meeting time (#1450)\n\nLooks like the doc in the repo was outdated.",
          "timestamp": "2025-11-19T00:42:26Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/a2fc3baf6988b2014a28f12006b56757c82ebc37"
        },
        "date": 1763634650666,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 44.351472180157614,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 46.59556031866347,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 30.891015625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 31.10546875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 13229611.846673166,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 12708412.728462745,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 999000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 99.9000015258789,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 0.06300656734044265,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 0.07846376255990106,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.53203125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.703125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 1000000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 1000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 58811.56253830586,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 4795.763494241108,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
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
          "id": "817117a937ed1eff8ddac54beec6417791f83562",
          "message": "Initial API & structure of columnar query engine (#1453)\n\nCloses #1417 & #1409 \n\nDuring last night's SIG meeting for the query/transform engine working\ngroup, there was some consensus that having our Pipelines made up of\nmultiple stages (instead of a single DataFusion `ExecutionPlan`), seemed\nlike a good option. This gives me confidence that the plan laid out in\n#1417 is probably workable - so this PR implements what's spec'd out\nthere with some modifications based on feedback on the issue.\n\nAdds the initial shell of the columnar query engine including the:\n- `Pipeline` - top-level API for invoking an OPL/KQL pipeline on OTAP\nData\n- `PlannedPipeline` - internal data structure used by the `Pipeline`\ncontaining the stages and state needed for adapting to changing OTAP\nbatch schemas\n- `PipelineStage` - trait implemented by a stage in the pipeline to\ntransform OTAP batch\n- `DataFusionPipelineStage` - an implementation of `PipelineStage`\nbacked by a DataFusion `ExecutionPlan`\n- `PipelinePlanner` - used to transform a the\n[`PipelineExpression`](https://github.com/open-telemetry/otel-arrow/blob/a2fc3baf6988b2014a28f12006b56757c82ebc37/rust/experimental/query_engine/expressions/src/pipeline_expression.rs#L7)\ninto `PipelineStages` (not actually implemented)\n- `RecordBatchPartitionStream` - implementation of\n[`PartitionStream`](https://docs.rs/datafusion/latest/datafusion/physical_plan/streaming/trait.PartitionStream.html)\nused to shuffle the current `RecordBatch` for a given OTAP Payload Type\ninto `DataFusionPiplineStage`'s `ExecutionPlan`\n\nThis gives us a concrete base which we can iterate on as we implement\nthe rest of the columnar engine.",
          "timestamp": "2025-11-20T17:48:00Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/817117a937ed1eff8ddac54beec6417791f83562"
        },
        "date": 1763720214566,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 44.82034010978036,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 46.37252004629273,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 32.828515625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 33.0234375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 13264534.652903248,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 12701297.26290914,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 999000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 99.9000015258789,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 0.03321464871564784,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 0.054574925698280555,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 27.04375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 27.171875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 1000000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 1000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 58821.617934108246,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 4796.622141476409,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
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
          "id": "07cf17a583364c4b356724a1fb18cde6c2c0b0d6",
          "message": "Use otelbot app token for Tidy push (#1460)\n\nAnother attempt to get solution working for #1456.\n\nIt *almost* worked as expected in #1458 but I forgot an earlier lesson\nfrom\n[prepare-release](https://github.com/open-telemetry/otel-arrow/blob/382c61d4b35eaf8c4753b046d57aa7bc70d3b0e6/.github/workflows/prepare-release.yml#L133-L134)\nthat we have to inject the special otelbot app token as an environment\nvariable in the task that does the Git action. Otherwise, regular PR\nworkflows are not triggered.\n\nI've seen some evidence that we should opt out of persisting regular git\ncreds from `checkout` in order for the `auth setup-git` to work. Would\nlike to try this configuration.",
          "timestamp": "2025-11-21T23:08:20Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/07cf17a583364c4b356724a1fb18cde6c2c0b0d6"
        },
        "date": 1763806555936,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 44.2622178650799,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 46.68682504439126,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 29.391796875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 29.55078125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 13247322.693869123,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 12726003.109319791,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 999000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 99.9000015258789,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 0.05635734745714665,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 0.08239859572815533,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 23.4609375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.54296875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 1000000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 1000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 58824.59171530414,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 4781.60375190079,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
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
          "id": "07cf17a583364c4b356724a1fb18cde6c2c0b0d6",
          "message": "Use otelbot app token for Tidy push (#1460)\n\nAnother attempt to get solution working for #1456.\n\nIt *almost* worked as expected in #1458 but I forgot an earlier lesson\nfrom\n[prepare-release](https://github.com/open-telemetry/otel-arrow/blob/382c61d4b35eaf8c4753b046d57aa7bc70d3b0e6/.github/workflows/prepare-release.yml#L133-L134)\nthat we have to inject the special otelbot app token as an environment\nvariable in the task that does the Git action. Otherwise, regular PR\nworkflows are not triggered.\n\nI've seen some evidence that we should opt out of persisting regular git\ncreds from `checkout` in order for the `auth setup-git` to work. Would\nlike to try this configuration.",
          "timestamp": "2025-11-21T23:08:20Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/07cf17a583364c4b356724a1fb18cde6c2c0b0d6"
        },
        "date": 1763892947254,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 44.29768823622336,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 46.60007189928169,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 29.2421875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 29.4765625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 13183750.511963971,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 12666202.337823473,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 999000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 99.9000015258789,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 0.0478078826379926,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 0.06923361474711487,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 28.998046875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 29.1328125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 1000000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 1000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 59122.16683245285,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 4822.178264614517,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
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
          "id": "07cf17a583364c4b356724a1fb18cde6c2c0b0d6",
          "message": "Use otelbot app token for Tidy push (#1460)\n\nAnother attempt to get solution working for #1456.\n\nIt *almost* worked as expected in #1458 but I forgot an earlier lesson\nfrom\n[prepare-release](https://github.com/open-telemetry/otel-arrow/blob/382c61d4b35eaf8c4753b046d57aa7bc70d3b0e6/.github/workflows/prepare-release.yml#L133-L134)\nthat we have to inject the special otelbot app token as an environment\nvariable in the task that does the Git action. Otherwise, regular PR\nworkflows are not triggered.\n\nI've seen some evidence that we should opt out of persisting regular git\ncreds from `checkout` in order for the `auth setup-git` to work. Would\nlike to try this configuration.",
          "timestamp": "2025-11-21T23:08:20Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/07cf17a583364c4b356724a1fb18cde6c2c0b0d6"
        },
        "date": 1763979462551,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 44.48387649837892,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 47.027087157894734,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 29.399609375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 29.6796875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 13181548.10928154,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 12623603.134150196,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 999000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 99.9000015258789,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 0.0480548716227815,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 0.0662127323247916,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 27.433203125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 27.5390625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 1000000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 1000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 59110.92232179704,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 4821.239115968015,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
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
          "id": "0dc05d926ef58004a70332e3cb286f1c7825de7d",
          "message": "[otap-dataflow benchmark] filter processor benchmarks + internal telemetry (#1448)\n\nAdd filter processor scenarios to the nightly benchmark suite\nCollect internal metrics inside the filter processor tracking, number of\nsignals before and after the filtering\n\n\n```rust\n/// Pdata-oriented metrics for the OTAP FilterProcessor\n#[metric_set(name = \"filter.processor.pdata.metrics\")]\n#[derive(Debug, Default, Clone)]\npub struct FilterPdataMetrics {\n    /// Number of log signals consumed\n    #[metric(unit = \"{log}\")]\n    pub log_signals_consumed: Counter<u64>,\n    /// Number of span signals consumed\n    #[metric(unit = \"{span}\")]\n    pub span_signals_consumed: Counter<u64>,\n\n    /// Number of log signals sent\n    #[metric(unit = \"{log}\")]\n    pub log_signals_sent: Counter<u64>,\n    /// Number of span signals sent\n    #[metric(unit = \"{span}\")]\n    pub span_signals_sent: Counter<u64>,\n}\n\n```\n\n---------\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2025-11-25T03:30:16Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/0dc05d926ef58004a70332e3cb286f1c7825de7d"
        },
        "date": 1764066302278,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 13.746887627166524,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 14.701689265239898,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 36.900390625,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 39.21875,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 234416.17017343215,
            "unit": "bits/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5867157.9514379855,
            "unit": "bits/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 12.289535593061403,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 13.068906898478733,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 35.418359375,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 37.43359375,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 207266.8645998687,
            "unit": "bits/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5764919.593433067,
            "unit": "bits/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 6000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.25,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 44.672213206319604,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 46.91575048605208,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 29.15703125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 29.36328125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 2394000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 13201882.309545172,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 12675712.661732089,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 999000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 99.9000015258789,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 0.04238728138931516,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 0.06071083468708904,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 23.68125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.7578125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 1000000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 1000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 58839.46817612663,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 4799.552620330536,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
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
          "id": "261dd531b0111ca72990d1aec9f69cb25160e1ce",
          "message": "OTLP Exporter Optimizations (#1474)\n\nSupport for multiple simultaneous client connections to improve\nthroughput and the ack/nack system.\n\n---------\n\nCo-authored-by: Joshua MacDonald <jmacd@users.noreply.github.com>",
          "timestamp": "2025-11-25T23:19:54Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/261dd531b0111ca72990d1aec9f69cb25160e1ce"
        },
        "date": 1764152516664,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 995000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 99.5,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 0.03240641175009484,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 0.04136765088207985,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.3890625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.484375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 1000000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 5000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 58805.81159417456,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 4779.441118828777,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 43.15419653464353,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 43.847920992555835,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 33.705859375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 36.98046875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 14389261.982563794,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13758191.333441079,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 11.856838005969486,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 12.114461054424403,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 38.482421875,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 40.52734375,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 190025.50253328303,
            "unit": "bits/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5674983.515469199,
            "unit": "bits/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 9.060211864862397,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 9.562600691050584,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 34.412109375,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 37.13671875,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 209221.28912404313,
            "unit": "bits/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5700107.271274016,
            "unit": "bits/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
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
          "id": "fb0dcd74f000bd88a0813ff8990b307393a65b62",
          "message": "[query-engine] Expand expressions to support user-defined functions (#1478)\n\nRelates to #1479\n\n## Changes\n\n* Make it possible to declare and invoke user-defined functions in query\nexpression tree\n\n## Details\n\nImplementation and KQL parsing will come as follow-ups.",
          "timestamp": "2025-11-26T18:08:37Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/fb0dcd74f000bd88a0813ff8990b307393a65b62"
        },
        "date": 1764238885693,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 17.05077970838318,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 18.627272753196436,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 35.919140625,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 37.03125,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 214144.21614473936,
            "unit": "bits/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5799026.770618999,
            "unit": "bits/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 13.251152860978827,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 13.779169116077645,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.346484375,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.15234375,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 196849.78565775958,
            "unit": "bits/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5794849.657250958,
            "unit": "bits/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 995000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 99.5,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 0.040696048063529854,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 0.059132512261580374,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 30.276171875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 30.5859375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 1000000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 5000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 59111.16542154682,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 4820.670333525153,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 43.56280377709475,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 46.02589229555126,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 34.10859375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 34.26171875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 14329007.33921421,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13791336.645145277,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
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
          "id": "fb0dcd74f000bd88a0813ff8990b307393a65b62",
          "message": "[query-engine] Expand expressions to support user-defined functions (#1478)\n\nRelates to #1479\n\n## Changes\n\n* Make it possible to declare and invoke user-defined functions in query\nexpression tree\n\n## Details\n\nImplementation and KQL parsing will come as follow-ups.",
          "timestamp": "2025-11-26T18:08:37Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/fb0dcd74f000bd88a0813ff8990b307393a65b62"
        },
        "date": 1764325276371,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 16.254802621342954,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 17.289083202725724,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 40.24609375,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.5078125,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 214160.99985531025,
            "unit": "bits/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5795807.0993703455,
            "unit": "bits/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 13.101162351444357,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 14.032187459727385,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 40.86796875,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 42.859375,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 198897.6674860423,
            "unit": "bits/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5811932.738830039,
            "unit": "bits/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 995000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 99.5,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 0.03997534329128496,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 0.052552722787542396,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 28.001953125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 28.10546875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 1000000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 5000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 58804.700850337904,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 4794.758452768769,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 43.07386615654505,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 43.69492352831941,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 34.991015625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 35.2734375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 14327590.924988415,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13795543.105416546,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
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
          "id": "fb0dcd74f000bd88a0813ff8990b307393a65b62",
          "message": "[query-engine] Expand expressions to support user-defined functions (#1478)\n\nRelates to #1479\n\n## Changes\n\n* Make it possible to declare and invoke user-defined functions in query\nexpression tree\n\n## Details\n\nImplementation and KQL parsing will come as follow-ups.",
          "timestamp": "2025-11-26T18:08:37Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/fb0dcd74f000bd88a0813ff8990b307393a65b62"
        },
        "date": 1764412051431,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 16.126417417196134,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 17.945965855161788,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 38.678515625,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 39.4921875,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 209874.09019324143,
            "unit": "bits/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5771462.040659498,
            "unit": "bits/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 43.4156699204953,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 44.060537605938755,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 34.4453125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 36.375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 14425483.80398797,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13807157.196075987,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 14.157161393187078,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 15.262294157650697,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 41.5125,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 44.51953125,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 198352.93740335447,
            "unit": "bits/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5791497.110859085,
            "unit": "bits/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 995000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 99.5,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 0.04026443115407032,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 0.06541189139011372,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.310546875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.34375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 1000000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 5000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 58828.26755209686,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 4796.684275498903,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
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
          "id": "fb0dcd74f000bd88a0813ff8990b307393a65b62",
          "message": "[query-engine] Expand expressions to support user-defined functions (#1478)\n\nRelates to #1479\n\n## Changes\n\n* Make it possible to declare and invoke user-defined functions in query\nexpression tree\n\n## Details\n\nImplementation and KQL parsing will come as follow-ups.",
          "timestamp": "2025-11-26T18:08:37Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/fb0dcd74f000bd88a0813ff8990b307393a65b62"
        },
        "date": 1764498059721,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 995000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 99.5,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 0.04662749152860179,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 0.06750775357809582,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 31.0890625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 31.21484375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 1000000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 5000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 58837.953344916445,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 4797.447001051496,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 13.458880579617515,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 14.816358270220448,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 41.100390625,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 42.41015625,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 194616.5055071979,
            "unit": "bits/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5848681.264504157,
            "unit": "bits/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 43.34650416659634,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 43.66531242323459,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 33.5515625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 34.56640625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 14437382.2528284,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13830460.249245385,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 18.17580731159785,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 18.8760708748068,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.106640625,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.2265625,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 214848.1698024809,
            "unit": "bits/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5879987.350168141,
            "unit": "bits/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
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
          "id": "fb0dcd74f000bd88a0813ff8990b307393a65b62",
          "message": "[query-engine] Expand expressions to support user-defined functions (#1478)\n\nRelates to #1479\n\n## Changes\n\n* Make it possible to declare and invoke user-defined functions in query\nexpression tree\n\n## Details\n\nImplementation and KQL parsing will come as follow-ups.",
          "timestamp": "2025-11-26T18:08:37Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/fb0dcd74f000bd88a0813ff8990b307393a65b62"
        },
        "date": 1764584549560,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 12.175971457117186,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 13.434306244484013,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.19609375,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 40.5234375,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 193261.2417335802,
            "unit": "bits/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5881412.113927194,
            "unit": "bits/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 43.560231043197675,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 43.99724449254078,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 36.0953125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 38.3515625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 14436332.527642291,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13863865.95726705,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 14.14692675747569,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 15.693897664379866,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 37.67109375,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 39.56640625,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 210370.12780972308,
            "unit": "bits/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5770563.23476807,
            "unit": "bits/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 995000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 99.5,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 0.048458166027773926,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 0.09472978986402966,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 24.030859375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 24.1015625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 1000000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 5000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 58836.28081294985,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 4797.165722008951,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
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
          "id": "e8cecc80e28d47b04d086c50dfcd1e4e6ac83fbb",
          "message": "Update all patch versions (#1487)\n\nThis PR contains the following updates:\n\n| Package | Change | Age | Confidence |\n|---|---|---|---|\n|\n[github.com/klauspost/compress](https://redirect.github.com/klauspost/compress)\n| `v1.18.1` -> `v1.18.2` |\n[![age](https://developer.mend.io/api/mc/badges/age/go/github.com%2fklauspost%2fcompress/v1.18.2?slim=true)](https://docs.renovatebot.com/merge-confidence/)\n|\n[![confidence](https://developer.mend.io/api/mc/badges/confidence/go/github.com%2fklauspost%2fcompress/v1.18.1/v1.18.2?slim=true)](https://docs.renovatebot.com/merge-confidence/)\n|\n| [pydantic](https://redirect.github.com/pydantic/pydantic)\n([changelog](https://docs.pydantic.dev/latest/changelog/)) | `==2.12.4`\n-> `==2.12.5` |\n[![age](https://developer.mend.io/api/mc/badges/age/pypi/pydantic/2.12.5?slim=true)](https://docs.renovatebot.com/merge-confidence/)\n|\n[![confidence](https://developer.mend.io/api/mc/badges/confidence/pypi/pydantic/2.12.4/2.12.5?slim=true)](https://docs.renovatebot.com/merge-confidence/)\n|\n\n---\n\n### Release Notes\n\n<details>\n<summary>klauspost/compress (github.com/klauspost/compress)</summary>\n\n###\n[`v1.18.2`](https://redirect.github.com/klauspost/compress/releases/tag/v1.18.2)\n\n[Compare\nSource](https://redirect.github.com/klauspost/compress/compare/v1.18.1...v1.18.2)\n\n##### What's Changed\n\n- Fix invalid encoding on level 9 with single value input by\n[@&#8203;klauspost](https://redirect.github.com/klauspost) in\n[#&#8203;1115](https://redirect.github.com/klauspost/compress/pull/1115)\n- flate: reduce stateless allocations by\n[@&#8203;RXamzin](https://redirect.github.com/RXamzin) in\n[#&#8203;1106](https://redirect.github.com/klauspost/compress/pull/1106)\n- build(deps): bump github/codeql-action from 3.30.5 to 4.31.2 in the\ngithub-actions group by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;1111](https://redirect.github.com/klauspost/compress/pull/1111)\n\n`v1.18.1` is marked \"retracted\" due to invalid flate/zip/gzip encoding.\n\n##### New Contributors\n\n- [@&#8203;RXamzin](https://redirect.github.com/RXamzin) made their\nfirst contribution in\n[#&#8203;1106](https://redirect.github.com/klauspost/compress/pull/1106)\n\n**Full Changelog**:\n<https://github.com/klauspost/compress/compare/v1.18.1...v1.18.2>\n\n</details>\n\n<details>\n<summary>pydantic/pydantic (pydantic)</summary>\n\n###\n[`v2.12.5`](https://redirect.github.com/pydantic/pydantic/releases/tag/v2.12.5):\n2025-11-26\n\n[Compare\nSource](https://redirect.github.com/pydantic/pydantic/compare/v2.12.4...v2.12.5)\n\n#### v2.12.5 (2025-11-26)\n\nThis is the fifth 2.12 patch release, addressing an issue with the\n`MISSING` sentinel and providing several documentation improvements.\n\nThe next 2.13 minor release will be published in a couple weeks, and\nwill include a new *polymorphic serialization* feature addressing\nthe remaining unexpected changes to the *serialize as any* behavior.\n\n- Fix pickle error when using `model_construct()` on a model with\n`MISSING` as a default value by\n[@&#8203;ornariece](https://redirect.github.com/ornariece) in\n[#&#8203;12522](https://redirect.github.com/pydantic/pydantic/pull/12522).\n- Several updates to the documentation by\n[@&#8203;Viicos](https://redirect.github.com/Viicos).\n\n**Full Changelog**:\n<https://github.com/pydantic/pydantic/compare/v2.12.4...v2.12.5>\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - \"before 8am every weekday\" (UTC),\nAutomerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n👻 **Immortal**: This PR will be recreated if closed unmerged. Get\n[config\nhelp](https://redirect.github.com/renovatebot/renovate/discussions) if\nthat's undesired.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0Mi4xOS45IiwidXBkYXRlZEluVmVyIjoiNDIuMTkuOSIsInRhcmdldEJyYW5jaCI6Im1haW4iLCJsYWJlbHMiOlsiZGVwZW5kZW5jaWVzIl19-->\n\n---------\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: otelbot <197425009+otelbot@users.noreply.github.com>",
          "timestamp": "2025-12-02T00:43:23Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/e8cecc80e28d47b04d086c50dfcd1e4e6ac83fbb"
        },
        "date": 1764670908160,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 12.928383612711396,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 13.671722116320383,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 41.665625,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 43.51953125,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 198148.13105612568,
            "unit": "bits/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5810271.22477187,
            "unit": "bits/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 43.56952820040784,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 44.66472192236599,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 32.412890625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 32.58984375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 14399554.396148164,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13824566.09005788,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 995000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 99.5,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 0.0544482521222095,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 0.07313018373517327,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 29.969921875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 30.03515625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 1000000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 5000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 59113.63309156635,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 4820.779584242848,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 15.857771849271545,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 17.046408530688158,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 34.69609375,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 36.9921875,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 213496.8094719728,
            "unit": "bits/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5771461.239546048,
            "unit": "bits/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
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
          "id": "89d92af81eab57dc4f7318396d950c313927457f",
          "message": "[otap-dataflow] fix exclude filter returning empty batch #1483 (#1504)\n\nFixes #1483\n\nAdds check for fields where an empty vec provided so nothing should be\nexcluded",
          "timestamp": "2025-12-02T20:03:36Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/89d92af81eab57dc4f7318396d950c313927457f"
        },
        "date": 1764757861720,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 16.12051693245539,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 16.70737204301075,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.48984375,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 43.046875,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 214382.85425452818,
            "unit": "bits/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5794395.469560363,
            "unit": "bits/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 995000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 99.5,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 0.050730693708342885,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 0.07205734909769758,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 30.9921875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 31.140625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 1000000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 5000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 58843.12864068661,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 4797.8438443033065,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 11.485472197317954,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 11.71136699155497,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.493359375,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.13671875,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2300000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 189446.52278875257,
            "unit": "bits/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5702475.523429902,
            "unit": "bits/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 43.43399060355704,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 44.12998223526686,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 28.1546875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 29.61328125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 14407122.182727376,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13870610.992524292,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
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
          "id": "7b23c8139eec1c18112a0a0506f692069e691ad6",
          "message": "Minor refactor otap_df_otap::otap_grpc::otlp::server_new (#1516)\n\nFrom https://github.com/lquerel/otel-arrow/pull/5\n\n---------\n\nCo-authored-by: querel <l.querel@f5.com>\nCo-authored-by: Laurent Quérel <laurent.querel@gmail.com>",
          "timestamp": "2025-12-03T22:02:16Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/7b23c8139eec1c18112a0a0506f692069e691ad6"
        },
        "date": 1764843722524,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 2300000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 100,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 0.033769291128819094,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 0.048261802455788094,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 10.666015625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 10.86328125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2300000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 45909.50332922976,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 63307.292417731136,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 100,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 0.015032207296528537,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 0.015601172365666433,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 10.59765625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 10.59765625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 45907.714462706834,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 63306.473461702684,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 11.359607746626358,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 12.26285178210956,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 34.0328125,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 36.62890625,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 195611.6714103928,
            "unit": "bits/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5780310.8990774555,
            "unit": "bits/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 11.663173843248979,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 12.769083696226122,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 34.92421875,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 36.65234375,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 211373.5442880856,
            "unit": "bits/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5771567.419684692,
            "unit": "bits/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
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
          "id": "5a5ca878dcea95586eb414b1b0fe4979319972d7",
          "message": "  Add TLS to OTLP/OTAP receivers (server-side, with reload) (#1510)\n\nThis PR adds server-side TLS for receivers, following the Go collector's\napproach. Plan is to submit changes incrementally to make the review\neasier and to get feedback on the implementation direction before\nraising further PRs.\n\n**TL;DR:** This enables TLS for receivers. mTLS and exporter client-side\nTLS are not part of this PR.\n\n  ### What's in this PR\n  **Server-side TLS for receivers:**\n  - OTLP and OTAP receivers can now accept TLS connections\n- Certificate and key configuration via `cert_file`/`key_file` or\n`cert_pem`/`key_pem`\n- Async certificate hot-reload without service restart or connection\ndrops\n\n  **Certificate reload behavior:**\n  - Similar to the Go collector's implementation\n  - File-based certificates are monitored via mtime\n- Reload triggered asynchronously when files change and\n`reload_interval` has elapsed\n- Active connections continue using the current certificate while reload\nhappens in background\n- Zero downtime during rotation assumes the new certificate is deployed\nbefore the old certificate expires, creating an overlap window where\nboth are valid.\nThe `reload_interval` should be set appropriately based on your\ncertificate rotation schedule (e.g., if certificates are rotated daily,\nuse `reload_interval:\n  \"5m\"` to ensure timely pickup).\n\n  **Implementation notes:**\n  - Uses `rustls` for TLS\n  - Certificate reload runs in spawned async tasks\n  - Compare-exchange used to ensure single reload task at a time\n- **NUMA note:** Each receiver creates its own TLS resolver instance\n(per-core design). Arc-wrapped atomics are used for local coordination\nbetween the resolver and its async reload task. No cross-NUMA sharing.\n\n  ### What's NOT in this PR\n\n  **Deferred to follow-up PRs:**\n  - mTLS (mutual TLS / client certificate verification)\n  - Client-side TLS for exporters\n  - Comprehensive benchmarks (planned after full TLS implementation)\n\n  ### Feature flag\n\nCurrently gated behind `experimental-tls` feature flag. This will be\nremoved once we're confident the implementation is stable and\nproduction-ready.\n\n  ### Testing\n\n  Manual E2E testing completed:\n  - Basic TLS handshake and data transmission\n  - Certificate hot-reload during active connections  \n  - Plaintext rejection when TLS is enabled\n  - Certificate chain validation\n  - Backward compatibility (plaintext mode works without feature flag)\n\n  Automated tests included:\n  - Unit tests for reload logic (`tls_utils::tests`)\n- Integration test for end-to-end certificate rotation\n(`tests/tls_reload.rs`)\n\n  ---\n\n  **Configuration example:**\n\n  ```yaml\n  receivers:\n    otlp:\n      config:\n        listening_addr: \"0.0.0.0:4319\"\n        tls:\n          cert_file: \"/path/to/cert.pem\"\n          key_file: \"/path/to/key.pem\"\n          reload_interval: \"5m\"  # Optional, defaults to 5 minutes\n\n \n```",
          "timestamp": "2025-12-04T22:58:53Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/5a5ca878dcea95586eb414b1b0fe4979319972d7"
        },
        "date": 1764930520857,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 6400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 100,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 0.03628644799581933,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 0.047651486130481945,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 0.03628644799581933,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 0.047651486130481945,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 10.702213541666667,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 10.86328125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 46065.39764939582,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 63503.92024643978,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 10.923023059217973,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 11.239306480749864,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 10.923023059217973,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 11.239306480749864,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 41.85182291666667,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 42.4453125,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6300000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 188327.6009113421,
            "unit": "bits/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5701540.961044814,
            "unit": "bits/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 16.209167398367715,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 18.089835210285806,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 16.209167398367715,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 18.089835210285806,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 36.42942708333333,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 39.03125,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6400000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 210208.50790394683,
            "unit": "bits/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5723708.218575765,
            "unit": "bits/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 6400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 100,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 0.026026368756041657,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 0.048619685670261946,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 0.026026368756041657,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 0.048619685670261946,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 8.837239583333334,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 8.9453125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 46062.86959524779,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 63489.45354969644,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
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
          "id": "dfca971bd964d35f4e00a57f179d004ea83b1d07",
          "message": "Batch processor Ack/Nack support (#1486)\n\nAdds routing state for inbound and outbound batches, calls notify_ack\nand notify_nack appropriately. This depends on behavior referred to as\n\"in-line\" delivery, which requires that points are not re-ordered by the\nbatcher; this property is required by the present algorithm to align Ack\nand Nack responses appropriately. This is tested lightly, now, in this\nPR. This requires more substantive testing in the lower-level library!\nThis only tests logs and traces, because metric batching has known\ncurrent defects.\n\nThe batch processor automatically tracks inbound and outbound context\nonly as needed, considering the whether the arriving data\n`has_subscribers()`, which will be determined by `wait_for_result: true`\nin the receiver.\n\nNew testing revealed a few more cases where protocol buffer form and\nOTAP-records-converted forms have insignificant differences: TraceID,\nSpanID, Resource, and scope presence information is lost, default values\nare filled in. The equivalence tests now canonicalize these.\n\nFixes #1326",
          "timestamp": "2025-12-06T01:23:12Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/dfca971bd964d35f4e00a57f179d004ea83b1d07"
        },
        "date": 1765016862521,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": -100000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 13.120525442887782,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 14.720328974260468,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 13.120525442887782,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 14.720328974260468,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 42.70130208333333,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 43.2109375,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 24107.583340829096,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 716771.2963512816,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": -100000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 42.486902651280104,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 43.38110093155747,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 42.486902651280104,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 43.38110093155747,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 37.031640625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 37.3125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_received_total",
            "value": 100000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2197575.8956314344,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2121967.31631403,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": -100000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 13.993583854355936,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 18.75497002785515,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 13.993583854355936,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 18.75497002785515,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 44.70690104166667,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 45.09375,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 26123.57411716559,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 713961.7660811321,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 0.041634486052052576,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 0.0857176532426374,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 0.041634486052052576,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 0.0857176532426374,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 32.93307291666667,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 33.046875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 411.61360086570016,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 605.4486254740057,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
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
          "id": "6564239839d48e773a1f0cc45c69064f60695217",
          "message": "Columnar query engine filter handle `== null` filters (#1538)\n\npart of #1508 \n\nHandles filter predicates like `severity_text == null` or\n`attributes[\"x\"] == null` (e.g. doesn't exist) when filtering using the\ncolumnar query engine.\n\nIt handles all the cases, including when the optional column is not\npresent, when then the ID column (used to join attributes) is not\npresent or null (meaning no attributes), and when attribute batches are\nentirely absent.\n\n---------\n\nCo-authored-by: Joshua MacDonald <jmacd@users.noreply.github.com>",
          "timestamp": "2025-12-06T13:43:58Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/6564239839d48e773a1f0cc45c69064f60695217"
        },
        "date": 1765103237923,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": -100000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 42.513614348971025,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 43.62704481341848,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 42.513614348971025,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 43.62704481341848,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 32.52174479166667,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 32.8359375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_received_total",
            "value": 100000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2195727.9973058845,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2121128.153972978,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": -100000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 16.90460477121298,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 18.889535158924208,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 16.90460477121298,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 18.889535158924208,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 40.03294270833333,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 43.0390625,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 26416.899692747807,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 722882.5490714575,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": -100000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 10.199267481369347,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 11.317920371459527,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 10.199267481369347,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 11.317920371459527,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 44.429296875,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 45.046875,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 23842.357219094763,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 716474.2904189081,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 0.0393764923810879,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 0.06579507001934236,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 0.0393764923810879,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 0.06579507001934236,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 31.087369791666667,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 31.16796875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 410.8832187707981,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 603.5391381823687,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
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
          "id": "6564239839d48e773a1f0cc45c69064f60695217",
          "message": "Columnar query engine filter handle `== null` filters (#1538)\n\npart of #1508 \n\nHandles filter predicates like `severity_text == null` or\n`attributes[\"x\"] == null` (e.g. doesn't exist) when filtering using the\ncolumnar query engine.\n\nIt handles all the cases, including when the optional column is not\npresent, when then the ID column (used to join attributes) is not\npresent or null (meaning no attributes), and when attribute batches are\nentirely absent.\n\n---------\n\nCo-authored-by: Joshua MacDonald <jmacd@users.noreply.github.com>",
          "timestamp": "2025-12-06T13:43:58Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/6564239839d48e773a1f0cc45c69064f60695217"
        },
        "date": 1765189776039,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": -100000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 42.76029916686761,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 43.62479817001545,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 42.76029916686761,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 43.62479817001545,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 30.7875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 31.23046875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_received_total",
            "value": 100000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2195664.2840499585,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2117483.669515832,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 0.045459720304746966,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 0.07605531247573946,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 0.045459720304746966,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 0.07605531247573946,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 33.10703125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 33.2421875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 409.96754877376657,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 603.8493841119431,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": -100000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 13.575966720117144,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 15.011155664141024,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 13.575966720117144,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 15.011155664141024,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 32.71315104166667,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 34.73828125,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 25985.937575275966,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 722983.0766158846,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": -100000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 11.409063734629385,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 12.34891816985368,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 11.409063734629385,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 12.34891816985368,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 40.66627604166667,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 43.96875,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 24140.643341829666,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 724505.520328071,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
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
          "id": "e8f26074e120041e52cf592b499b1694dfc185c2",
          "message": "Perf test - change loadgen template to allow null (#1582)\n\nOnce https://github.com/open-telemetry/otel-arrow/pull/1581 is merged,\nI'll be working on adding a new nightly test to stress the engine to its\nmax. This is a simple pre-req, to allow passing `null` to configuration,\nresulting in an outcome like below:\n\n```yaml\nconfig:\n      traffic_config:\n        max_batch_size: 1000\n        signals_per_second: null\n        metric_weight: 0\n        trace_weight: 0\n        log_weight: 100\n```",
          "timestamp": "2025-12-10T23:36:22Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/e8f26074e120041e52cf592b499b1694dfc185c2"
        },
        "date": 1765449292914,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 11.882441474069177,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 12.841419026309698,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 11.882441474069177,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 12.841419026309698,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.06302083333333,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.171875,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6400000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 26042.06415916754,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 722256.6568796084,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 995000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 99.5,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 0.043670983685033575,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 0.07323809377165294,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 0.043670983685033575,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 0.07323809377165294,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 30.151692708333332,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 30.22265625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 1000000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 5000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 10864.502424389126,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 670.3336236769484,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 43.147652205733365,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 43.72689025339016,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 43.147652205733365,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 43.72689025339016,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 31.853255208333334,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 32.92578125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 6400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2206606.559302179,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2126276.841051322,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 11.981509462390799,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 14.302079109461966,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 11.981509462390799,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 14.302079109461966,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 42.02174479166667,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 42.8671875,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6400000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 24108.43866654357,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 723627.8955544623,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
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
          "id": "91a5aa4b5bb2729e869ab32a8e9263a42462aa20",
          "message": "Run perf test always but on cheap GH runners (#1604)\n\nFor catching issues like this\nhttps://github.com/open-telemetry/otel-arrow/pull/1602#issuecomment-3644189168,\nwe can run perf test in every PR. To save the dedicated hardware, we'll\njust run on the normal GH runners. if the label is applied\n(maintainers/approvers do this), then we run on the Oracle dedicated\nhardware.",
          "timestamp": "2025-12-12T00:30:07Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/91a5aa4b5bb2729e869ab32a8e9263a42462aa20"
        },
        "date": 1765535686660,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 16.139465888008715,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 17.41925439358915,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 16.139465888008715,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 17.41925439358915,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 38.479166666666664,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.23828125,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6400000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 26245.745666051447,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 722397.7885647409,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 11.108910389816181,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 12.022422308288148,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 11.108910389816181,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 12.022422308288148,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 41.91979166666667,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 43.09375,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6400000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 23830.316978852148,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 715718.7281755712,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 995000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 99.5,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 0.04728239017256639,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 0.08756103831891224,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 0.04728239017256639,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 0.08756103831891224,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 22.279947916666668,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 22.45703125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 1000000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 5000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 10858.422359054322,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 669.2922098201886,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 42.194744453004766,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 43.10188018188825,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 42.194744453004766,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 43.10188018188825,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 36.163151041666666,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 36.4921875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 6400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2199278.7380738826,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2123080.1790428953,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
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
          "id": "a0c0fb94ed9c74a9ec6de13c8b11b077c69f92ee",
          "message": "[otap-df-quiver] WAL Refactoring (#1616)\n\n- Updated terminology from \"checkpoint\" to \"cursor\" throughout the WAL\nwriter implementation for clarity.\n- Update WAL Header and Cursor (formerly 'checkpoint') file formats to\nuse variable length headers for better forward compatibility",
          "timestamp": "2025-12-13T00:30:33Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/a0c0fb94ed9c74a9ec6de13c8b11b077c69f92ee"
        },
        "date": 1765622074831,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 995000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 99.5,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 0.046226160601488125,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 0.06609667131134851,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 0.046226160601488125,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 0.06609667131134851,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 41.89114583333333,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 43.9453125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 1000000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 5000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 16666.263898622452,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 83.33131949311225,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00145,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 16819.851903441828,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 737.6499051860488,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 18.22083831816866,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 20.000869183499727,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 18.22083831816866,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 20.000869183499727,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 49.489453125,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 52.12890625,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6400000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 106664.76447836681,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106664.76447836681,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00107,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 33190.37622709948,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 722695.076722017,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 43.008366137634106,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 44.50407693465407,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 43.008366137634106,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 44.50407693465407,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 47.85390625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 48.609375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 6400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 106664.97069363264,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106664.97069363264,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000954,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2185899.3140817997,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2097108.6320973784,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 13.898691299021406,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 15.411722850489054,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 13.898691299021406,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 15.411722850489054,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 49.78151041666667,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 51.453125,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6400000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 106664.69514755136,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106664.69514755136,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001109,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 30070.341174840036,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 723087.583786515,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
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
          "id": "a0c0fb94ed9c74a9ec6de13c8b11b077c69f92ee",
          "message": "[otap-df-quiver] WAL Refactoring (#1616)\n\n- Updated terminology from \"checkpoint\" to \"cursor\" throughout the WAL\nwriter implementation for clarity.\n- Update WAL Header and Cursor (formerly 'checkpoint') file formats to\nuse variable length headers for better forward compatibility",
          "timestamp": "2025-12-13T00:30:33Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/a0c0fb94ed9c74a9ec6de13c8b11b077c69f92ee"
        },
        "date": 1765708458921,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 995000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 99.5,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 0.05126272229984548,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 0.07504869591993842,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 0.05126272229984548,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 0.07504869591993842,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 41.86510416666667,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 43.890625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 1000000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 5000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 16666.285842035177,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 83.33142921017588,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001371,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 16522.369802999965,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 726.0029720572285,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 18.687842900628297,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 20.895704470879036,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 18.687842900628297,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 20.895704470879036,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 53.026171875,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 55.7109375,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6400000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 106664.18316893523,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106664.18316893523,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001397,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 33188.00836595931,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 722137.08964338,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 42.65572293559397,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 43.40856657686948,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 42.65572293559397,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 43.40856657686948,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 46.84205729166667,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 48.5234375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 6400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 106664.0178435569,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106664.0178435569,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00149,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2200596.8542312696,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2107795.479411888,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 13.82593049474819,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 15.206027127003699,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 13.82593049474819,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 15.206027127003699,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 52.83450520833333,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 55.40625,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6400000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 106664.2649429677,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106664.2649429677,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001351,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 29928.156240810975,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 723418.8683961666,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
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
          "id": "a0c0fb94ed9c74a9ec6de13c8b11b077c69f92ee",
          "message": "[otap-df-quiver] WAL Refactoring (#1616)\n\n- Updated terminology from \"checkpoint\" to \"cursor\" throughout the WAL\nwriter implementation for clarity.\n- Update WAL Header and Cursor (formerly 'checkpoint') file formats to\nuse variable length headers for better forward compatibility",
          "timestamp": "2025-12-13T00:30:33Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/a0c0fb94ed9c74a9ec6de13c8b11b077c69f92ee"
        },
        "date": 1765795020839,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 995000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 99.5,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 0.06312783305638295,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 0.11336733559740661,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 0.06312783305638295,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 0.11336733559740661,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 42.008723958333334,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 43.93359375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 1000000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 5000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 16666.240566449516,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 83.33120283224758,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001534,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 16826.647560153873,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 735.7203919304841,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 42.67884573294188,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 43.80249502425502,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 42.67884573294188,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 43.80249502425502,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 47.116796875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 48.28515625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 6400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 106664.16539198822,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106664.16539198822,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001407,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2192172.5561681213,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2105007.823925311,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 18.49647804191516,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 20.068579529120196,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 18.49647804191516,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 20.068579529120196,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 53.156901041666664,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 55.7734375,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6400000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 106664.01428817803,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106664.01428817803,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001492,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 33083.195272584446,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 722411.4064923011,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 11.508828492199882,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 12.526225630525825,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 11.508828492199882,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 12.526225630525825,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 41.55247395833333,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 43.28515625,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6400000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 106664.35205022717,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106664.35205022717,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001302,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 30439.447102984494,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 722994.7112887116,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
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
          "id": "102d2d694a9089a3f96a203be0532024658e0fe0",
          "message": "chore(deps): update rust crate rcgen to 0.14 (#1625)\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n|---|---|---|---|\n| [rcgen](https://redirect.github.com/rustls/rcgen) |\nworkspace.dependencies | minor | `0.13` -> `0.14` |\n\n---\n\n### Release Notes\n\n<details>\n<summary>rustls/rcgen (rcgen)</summary>\n\n###\n[`v0.14.6`](https://redirect.github.com/rustls/rcgen/releases/tag/v0.14.6):\n0.14.6\n\n[Compare\nSource](https://redirect.github.com/rustls/rcgen/compare/v0.14.5...v0.14.6)\n\n#### What's Changed\n\n- Use private cfg for docs.rs-like builds by\n[@&#8203;ctz](https://redirect.github.com/ctz) in\n[#&#8203;384](https://redirect.github.com/rustls/rcgen/pull/384)\n- Expand rustdoc for CertificateSigningRequestParams::from\\_der by\n[@&#8203;dwhjames](https://redirect.github.com/dwhjames) in\n[#&#8203;386](https://redirect.github.com/rustls/rcgen/pull/386)\n- Group imports by\n[@&#8203;iamjpotts](https://redirect.github.com/iamjpotts) in\n[#&#8203;381](https://redirect.github.com/rustls/rcgen/pull/381)\n- examples: add signing new cert using existing ca pem files by\n[@&#8203;iamjpotts](https://redirect.github.com/iamjpotts) in\n[#&#8203;379](https://redirect.github.com/rustls/rcgen/pull/379)\n- Tweak CSR parsing errors/documentation by\n[@&#8203;djc](https://redirect.github.com/djc) in\n[#&#8203;390](https://redirect.github.com/rustls/rcgen/pull/390)\n- Rename invalid CSR signature error variant by\n[@&#8203;djc](https://redirect.github.com/djc) in\n[#&#8203;393](https://redirect.github.com/rustls/rcgen/pull/393)\n- chore: fix some typos in comments by\n[@&#8203;black5box](https://redirect.github.com/black5box) in\n[#&#8203;395](https://redirect.github.com/rustls/rcgen/pull/395)\n- ci: sync cargo-check-external-types nightly by\n[@&#8203;cpu](https://redirect.github.com/cpu) in\n[#&#8203;399](https://redirect.github.com/rustls/rcgen/pull/399)\n- Forward selected crypto backend to x509-parser by\n[@&#8203;djc](https://redirect.github.com/djc) in\n[#&#8203;398](https://redirect.github.com/rustls/rcgen/pull/398)\n\n###\n[`v0.14.5`](https://redirect.github.com/rustls/rcgen/releases/tag/v0.14.5):\n0.14.5\n\n[Compare\nSource](https://redirect.github.com/rustls/rcgen/compare/v0.14.4...v0.14.5)\n\nImplement SigningKey for `&impl SigningKey` to make `Issuer` more\nbroadly useful.\n\n#### What's Changed\n\n- Forward signing and public key data through references by\n[@&#8203;djc](https://redirect.github.com/djc) in\n[#&#8203;380](https://redirect.github.com/rustls/rcgen/pull/380)\n\n###\n[`v0.14.4`](https://redirect.github.com/rustls/rcgen/releases/tag/v0.14.4):\n0.14.4\n\n[Compare\nSource](https://redirect.github.com/rustls/rcgen/compare/v0.14.3...v0.14.4)\n\n#### What's Changed\n\n- Upgrade botan to 0.12 by\n[@&#8203;djc](https://redirect.github.com/djc) in\n[#&#8203;377](https://redirect.github.com/rustls/rcgen/pull/377)\n- Upgrade x509-parser to 0.18 by\n[@&#8203;djc](https://redirect.github.com/djc) in\n[#&#8203;376](https://redirect.github.com/rustls/rcgen/pull/376)\n- Add unstable support for ML-DSA algorithms by\n[@&#8203;djc](https://redirect.github.com/djc) in\n[#&#8203;374](https://redirect.github.com/rustls/rcgen/pull/374)\n\n###\n[`v0.14.3`](https://redirect.github.com/rustls/rcgen/releases/tag/v0.14.3):\n0.14.3\n\n[Compare\nSource](https://redirect.github.com/rustls/rcgen/compare/v0.14.2...v0.14.3)\n\n#### What's Changed\n\n- docs: fix typo in `PKCS_RSA_SHA384` doc comment by\n[@&#8203;Bravo555](https://redirect.github.com/Bravo555) in\n[#&#8203;367](https://redirect.github.com/rustls/rcgen/pull/367)\n- Fix regression in key usage purpose encoding by\n[@&#8203;djc](https://redirect.github.com/djc) in\n[#&#8203;369](https://redirect.github.com/rustls/rcgen/pull/369)\n\n###\n[`v0.14.2`](https://redirect.github.com/rustls/rcgen/releases/tag/v0.14.2):\n0.14.2\n\n[Compare\nSource](https://redirect.github.com/rustls/rcgen/compare/v0.14.1...v0.14.2)\n\n- Add a `CertifiedIssuer` type (see\n[#&#8203;363](https://redirect.github.com/rustls/rcgen/issues/363))\n\n#### What's changed\n\n- Add a CertifiedIssuer by\n[@&#8203;djc](https://redirect.github.com/djc) in\n[#&#8203;363](https://redirect.github.com/rustls/rcgen/pull/363)\n- Provide a non-owning constructor for `Issuer` by\n[@&#8203;p-avital](https://redirect.github.com/p-avital) in\n[#&#8203;362](https://redirect.github.com/rustls/rcgen/pull/362)\n- Allow access to the CertifiedIssuer's Certificate by\n[@&#8203;djc](https://redirect.github.com/djc) in\n[#&#8203;364](https://redirect.github.com/rustls/rcgen/pull/364)\n\n###\n[`v0.14.1`](https://redirect.github.com/rustls/rcgen/releases/tag/v0.14.1):\n0.14.1\n\n[Compare\nSource](https://redirect.github.com/rustls/rcgen/compare/v0.14.0...v0.14.1)\n\nDeclare 1.71 `rust-version` and check MSRV in CI.\n\n#### What's Changed\n\n- Check MSRV in CI by [@&#8203;djc](https://redirect.github.com/djc) in\n[#&#8203;361](https://redirect.github.com/rustls/rcgen/pull/361)\n\n###\n[`v0.14.0`](https://redirect.github.com/rustls/rcgen/releases/tag/v0.14.0):\n0.14.0\n\n[Compare\nSource](https://redirect.github.com/rustls/rcgen/compare/v0.13.3...v0.14.0)\n\n0.14.0 contains a number of potentially breaking API changes, though\nhopefully the rate of API change should slow down after this. Here is a\nsummary of the most noticeable changes you might run into:\n\n- `signed_by()` methods now take a reference to an `&Issuer` type that\ncontains both the issuer's relevant certificate parameters and the\nsigning key (see\n[#&#8203;356](https://redirect.github.com/rustls/rcgen/issues/356)). The\n`from_ca_cert_der()` and `from_ca_cert_pem()` constructors that were\npreviously attached to `CertificateParams` are now attached to `Issuer`\ninstead, removing a number of documented caveats.\n- The `RemoteKeyPair` trait is now called `SigningKey` and instead of\n`KeyPair` being an enum that contains a `Remote` variant, that variant\nhas been removed in favor of `KeyPair` implementing the trait (see\n[#&#8203;328](https://redirect.github.com/rustls/rcgen/issues/328)). To\nalign with this change, the `CertifiedKey::key_pair` field is now called\n`signing_key`, and `CertifiedKey` is generic over the signing key type.\n- The `KeyPair::public_key_der()` method has moved to\n`PublicKeyData::subject_public_key_info()` (see\n[#&#8203;328](https://redirect.github.com/rustls/rcgen/issues/328)).\n- Output types like `Certificate` no longer contain their originating\n`CertificateParams`. Instead, `signed_by()` and `self_signed()` now take\n`&self`, allowing the caller to retain access to the input parameters\n(see\n[#&#8203;328](https://redirect.github.com/rustls/rcgen/issues/328)). In\norder to make this possible, `Certificate::key_identifier()` can now be\naccessed via `CertificateParams` directly.\n- String types have been moved into a module (see\n[#&#8203;329](https://redirect.github.com/rustls/rcgen/issues/329)).\n\n#### What's Changed\n\n- Revert impl AsRef issuer by\n[@&#8203;audunhalland](https://redirect.github.com/audunhalland) in\n[#&#8203;325](https://redirect.github.com/rustls/rcgen/pull/325)\n- Move string types to separate module by\n[@&#8203;est31](https://redirect.github.com/est31) in\n[#&#8203;329](https://redirect.github.com/rustls/rcgen/pull/329)\n- Unbundle params from output types by\n[@&#8203;djc](https://redirect.github.com/djc) in\n[#&#8203;328](https://redirect.github.com/rustls/rcgen/pull/328)\n- Deduplicate Issuer construction by\n[@&#8203;djc](https://redirect.github.com/djc) in\n[#&#8203;332](https://redirect.github.com/rustls/rcgen/pull/332)\n- Extract write\\_extensions() method, reducing rightward drift by\n[@&#8203;djc](https://redirect.github.com/djc) in\n[#&#8203;333](https://redirect.github.com/rustls/rcgen/pull/333)\n- Update 0.12-to-0.13.md by\n[@&#8203;Alirexaa](https://redirect.github.com/Alirexaa) in\n[#&#8203;338](https://redirect.github.com/rustls/rcgen/pull/338)\n- Distribute methods for parsing params elements from x509 by\n[@&#8203;djc](https://redirect.github.com/djc) in\n[#&#8203;336](https://redirect.github.com/rustls/rcgen/pull/336)\n- Eagerly derive Clone, Copy, where possible by\n[@&#8203;lvkv](https://redirect.github.com/lvkv) in\n[#&#8203;341](https://redirect.github.com/rustls/rcgen/pull/341)\n- Updated `.gitignore` to be more specific by\n[@&#8203;Rynibami](https://redirect.github.com/Rynibami) in\n[#&#8203;342](https://redirect.github.com/rustls/rcgen/pull/342)\n- Eagerly implemented `Debug` trait by\n[@&#8203;Rynibami](https://redirect.github.com/Rynibami) in\n[#&#8203;343](https://redirect.github.com/rustls/rcgen/pull/343)\n- Minor tweaks to Debug impls and other style improvements by\n[@&#8203;djc](https://redirect.github.com/djc) in\n[#&#8203;348](https://redirect.github.com/rustls/rcgen/pull/348)\n- tests: only test against openssl on Unix by\n[@&#8203;djc](https://redirect.github.com/djc) in\n[#&#8203;350](https://redirect.github.com/rustls/rcgen/pull/350)\n- Eagerly implemented `PartialEq` and `Eq` traits by\n[@&#8203;Rynibami](https://redirect.github.com/Rynibami) in\n[#&#8203;344](https://redirect.github.com/rustls/rcgen/pull/344)\n- Use Issuer directly in the public API by\n[@&#8203;djc](https://redirect.github.com/djc) in\n[#&#8203;356](https://redirect.github.com/rustls/rcgen/pull/356)\n- Tweak docstring for PublicKeyData::subject\\_public\\_key\\_info() by\n[@&#8203;djc](https://redirect.github.com/djc) in\n[#&#8203;358](https://redirect.github.com/rustls/rcgen/pull/358)\n\n###\n[`v0.13.3`](https://redirect.github.com/rustls/rcgen/releases/tag/v0.13.3):\n0.13.3\n\n[Compare\nSource](https://redirect.github.com/rustls/rcgen/compare/v0.13.2...v0.13.3)\n\nThis release was yanked due to\n[#&#8203;324](https://redirect.github.com/rustls/rcgen/issues/324)\n\n#### What's Changed\n\n- Update dependencies by [@&#8203;djc](https://redirect.github.com/djc)\nin [#&#8203;305](https://redirect.github.com/rustls/rcgen/pull/305)\n- Add link to GitHub releases by\n[@&#8203;djc](https://redirect.github.com/djc) in\n[#&#8203;304](https://redirect.github.com/rustls/rcgen/pull/304)\n- change signature of signed\\_by to accept \\&impl\nAsRef<CertificateParams> issuer by\n[@&#8203;audunhalland](https://redirect.github.com/audunhalland) in\n[#&#8203;307](https://redirect.github.com/rustls/rcgen/pull/307)\n- Clarify CertificateParams::signed\\_by() docs by\n[@&#8203;djc](https://redirect.github.com/djc) in\n[#&#8203;308](https://redirect.github.com/rustls/rcgen/pull/308)\n- refactor: Generalize csr/crl signed\\_by to take \\&impl AsRef issuer by\n[@&#8203;audunhalland](https://redirect.github.com/audunhalland) in\n[#&#8203;312](https://redirect.github.com/rustls/rcgen/pull/312)\n- Fix: mark SAN as critical when subject is empty by\n[@&#8203;howardjohn](https://redirect.github.com/howardjohn) in\n[#&#8203;311](https://redirect.github.com/rustls/rcgen/pull/311)\n- Elide private key in KeyPair Debug impl by\n[@&#8203;lvkv](https://redirect.github.com/lvkv) in\n[#&#8203;314](https://redirect.github.com/rustls/rcgen/pull/314)\n- derive Debug for non-sensitive struct types by\n[@&#8203;cpu](https://redirect.github.com/cpu) in\n[#&#8203;316](https://redirect.github.com/rustls/rcgen/pull/316)\n- update LICENSE by\n[@&#8203;jasmyhigh](https://redirect.github.com/jasmyhigh) in\n[#&#8203;318](https://redirect.github.com/rustls/rcgen/pull/318)\n- Make `Certificate` cloneable (derive `Clone`) by\n[@&#8203;MadLittleMods](https://redirect.github.com/MadLittleMods) in\n[#&#8203;319](https://redirect.github.com/rustls/rcgen/pull/319)\n- Update dependencies by [@&#8203;djc](https://redirect.github.com/djc)\nin [#&#8203;321](https://redirect.github.com/rustls/rcgen/pull/321)\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - \"before 8am on Monday\" (UTC),\nAutomerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0Mi40Mi4yIiwidXBkYXRlZEluVmVyIjoiNDIuNDIuMiIsInRhcmdldEJyYW5jaCI6Im1haW4iLCJsYWJlbHMiOlsiZGVwZW5kZW5jaWVzIl19-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2025-12-15T18:32:00Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/102d2d694a9089a3f96a203be0532024658e0fe0"
        },
        "date": 1765881131873,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 15.22427750155554,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 16.168457482214663,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 15.22427750155554,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 16.168457482214663,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 44.55247395833333,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 45.6171875,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6400000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 106663.94495833782,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106663.94495833782,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001531,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 32958.83064630025,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 721732.0135706756,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 995000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 99.5,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 0.057512055886018223,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 0.12108819753086421,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 0.057512055886018223,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 0.12108819753086421,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 31.559244791666668,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 31.8046875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 1000000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 5000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 16666.333062233203,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 83.33166531116602,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001201,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 16514.725804861155,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 722.4855159211024,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 12.147003726424744,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 13.787845265171809,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 12.147003726424744,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 13.787845265171809,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 41.44544270833333,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 43.34375,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6400000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 106664.89069623657,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106664.89069623657,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000999,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 30658.93879007775,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 722885.725982665,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 42.642336923491555,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 43.53091973840117,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 42.642336923491555,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 43.53091973840117,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 36.78658854166667,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 38.46484375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 6400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 106664.000066665,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106664.000066665,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.0015,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2179118.6576303304,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2087736.6737949864,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
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
          "id": "865730028aeacfbe679d94d531d15cb2257aeb44",
          "message": "Adds OTAP Dataflow processor that uses columnar query engine (#1638)\n\ncloses #1628 \n\nAdds a processor implementation to OTAP Dataflow that uses the columnar\nquery engine to transform telemetry data.\n\nExample config:\n```yaml\nnodes:\n  transform:\n    kind: processor\n    plugin_urn: urn:otel:transform:processor\n    out_ports:\n      out_port:\n        destinations:\n        - exporter\n        dispatch_strategy: round_robin\n    config:\n      query: logs | where event_name == \"gen_ai.system.message\"\n```\n\n**I'm flexible on the name of this plugin and open to suggestions**. I\nhesitated between some alternative names:\n- \"\\<_language_\\> processor\" - where \"_language_\" = KQL/OTTL/ anything\nthat can be transformed into our expression AST. I was thinking this\ncould eventually accept different ways to express the transform so\npinning it to one language seemed too specific.\n- \"transform processor\" - this is a good name that expresses the purpose\nof the processor, but it's not really API compatible with Go collector's\n[transform\nprocessor](https://github.com/open-telemetry/opentelemetry-collector-contrib/tree/main/processor/transformprocessor)\nand I didn't want to cause confusion.\n\n---------\n\nCo-authored-by: Lalit Kumar Bhasin <lalit_fin@yahoo.com>",
          "timestamp": "2025-12-17T00:44:50Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/865730028aeacfbe679d94d531d15cb2257aeb44"
        },
        "date": 1765967531794,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 3.682105847049692,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 5.010182521163614,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 3.682105847049692,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 5.010182521163614,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 46.655598958333336,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 49.5546875,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6400000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 106664.53159829251,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106664.53159829251,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001201,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 14420.205266621775,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 349070.67120769084,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 39.05265507140949,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 45.62609674002008,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 39.05265507140949,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 45.62609674002008,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 74.09934895833334,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 84.54296875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 6400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 106663.98228977903,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106663.98228977903,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00151,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 1906895.0202643846,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1823875.5161263004,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 4.113620795544644,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 5.387560424181438,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 4.113620795544644,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 5.387560424181438,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 47.465755208333334,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 49.703125,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6400000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 106664.42671370567,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106664.42671370567,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00126,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 14564.641946696434,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 349164.6238121442,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 3250000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 98.48484802246094,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 0.061301843342209325,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 0.08441961330239753,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 0.061301843342209325,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 0.08441961330239753,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 55.653515625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 55.859375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 3300000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 50000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 54998.87985614693,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 833.3163614567717,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001222,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 16806.687326444877,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 736.1700380994857,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
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
          "id": "ac99143900169732ae2a7c90ed7a61cc4bd37471",
          "message": "[otap-df-quiver] Segment file reader/writer for Quiver durable storage layer. (#1643)\n\nImplements Segment file reader/writer for Quiver durable storage layer",
          "timestamp": "2025-12-17T23:42:12Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/ac99143900169732ae2a7c90ed7a61cc4bd37471"
        },
        "date": 1766053920738,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 4.065665253914415,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 5.254269015415602,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 4.065665253914415,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 5.254269015415602,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 47.74830729166667,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 50.015625,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6400000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 106664.59559576884,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106664.59559576884,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001165,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 14887.945907960293,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 349379.9813479271,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 38.924489094196865,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 41.010546481652796,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 38.924489094196865,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 41.010546481652796,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 68.97200520833333,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 82.25390625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6300000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 6300000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 104997.70055035794,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 104997.70055035794,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001314,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 1881956.7478679717,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1795396.7099775514,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 3.7086238328747427,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 4.944288797961862,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 3.7086238328747427,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 4.944288797961862,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 47.9703125,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 50.62890625,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108331.06379754677,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108331.06379754677,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001257,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 14518.40958709586,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 349245.9488053556,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 10000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.15625,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 32.103139743807255,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 33.253903884361065,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 32.103139743807255,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 33.253903884361065,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 58.985416666666666,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 63.19140625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 6390000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 106664.13872657885,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106497.47600981857,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001422,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2112016.058557422,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 352395.80597591697,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
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
          "id": "2c5e35748dc82b3cee511b767e9ca66ee66df7ff",
          "message": "Support deleting attributes in columnar query engine (#1654)\n\ncloses #1639 \n\nAdds the ability to delete attributes to the columnar query engine.\n\nWe now support queries like:\n```kql\nlogs | project-away attributes[\"x\"], attributes[\"y\"]\n```\n\n---------\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2025-12-19T03:20:40Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/2c5e35748dc82b3cee511b767e9ca66ee66df7ff"
        },
        "date": 1766140302600,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 43.014809351809966,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 43.86852070023238,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 43.014809351809966,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 43.86852070023238,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.08619791666667,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 40.5703125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108331.04213179224,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108331.04213179224,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001269,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2199167.642800773,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2104029.820470244,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 5000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.078125,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 45.32161073889031,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 46.52827981118936,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 45.32161073889031,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 46.52827981118936,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 37.73059895833333,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 38.703125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 6395000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 106664.41604748806,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106581.08447245097,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001266,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2378555.4753195993,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 740076.7194024723,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
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
          "id": "1efee73c2ef92e2d7c2957928945dd5ece06820d",
          "message": "Add memory info to SystemInformation log (#1666)",
          "timestamp": "2025-12-20T00:00:55Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/1efee73c2ef92e2d7c2957928945dd5ece06820d"
        },
        "date": 1766226689631,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 42.51670469615458,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 43.89732191124169,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 42.51670469615458,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 43.89732191124169,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.217578125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 40.53515625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108331.32378727708,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108331.32378727708,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001113,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2181915.091522161,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2083744.1098004752,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 45.14110623744655,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 46.71555794388189,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 45.14110623744655,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 46.71555794388189,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 38.7125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.1796875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108331.27684459457,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108331.27684459457,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001139,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2393432.2077423832,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 739361.2200583855,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
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
          "id": "1efee73c2ef92e2d7c2957928945dd5ece06820d",
          "message": "Add memory info to SystemInformation log (#1666)",
          "timestamp": "2025-12-20T00:00:55Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/1efee73c2ef92e2d7c2957928945dd5ece06820d"
        },
        "date": 1766313024159,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 42.45565161676152,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 43.249028454756385,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 42.45565161676152,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 43.249028454756385,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.877994791666666,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 40.87109375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108330.98977292125,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108330.98977292125,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001298,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2189808.0832882775,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2095777.7698422335,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 44.74694695045013,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 46.41778008818751,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 44.74694695045013,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 46.41778008818751,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 38.499088541666666,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.36328125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108329.01281120571,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108329.01281120571,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002393,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2392111.3519952325,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 740404.7349238412,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
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
          "id": "1efee73c2ef92e2d7c2957928945dd5ece06820d",
          "message": "Add memory info to SystemInformation log (#1666)",
          "timestamp": "2025-12-20T00:00:55Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/1efee73c2ef92e2d7c2957928945dd5ece06820d"
        },
        "date": 1766399478912,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 42.70961197957647,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 43.45967997530483,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 42.70961197957647,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 43.45967997530483,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.718359375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.1328125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108331.25698424113,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108331.25698424113,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00115,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2184542.762447253,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2089513.7684442697,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 4000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.0615384615957737,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 45.3652515725109,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 46.55437737960887,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 45.3652515725109,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 46.55437737960887,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 38.57643229166667,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.1640625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 6496000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108331.16670999913,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108264.50137663913,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.0012,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2390817.57933478,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 740861.4621544122,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
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
          "id": "9892ed711f59b7276e28cccc8b4e8ad0f9ebf884",
          "message": "Upgrade Collector dependencies to 0.142.0 and misc Go modules (#1682)\n\nSupersedes #1677, #1676, and #1675.\n\nTouched manually since already upgrading Collector dependencies.",
          "timestamp": "2025-12-22T21:24:59Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/9892ed711f59b7276e28cccc8b4e8ad0f9ebf884"
        },
        "date": 1766485923109,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 43.7846348725096,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 45.74688296479309,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 43.7846348725096,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 45.74688296479309,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.383984375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.06640625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108331.55850129989,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108331.55850129989,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000983,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2401044.6773184794,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 741529.0660246016,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 40.60932509402778,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 41.4839313258427,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 40.60932509402778,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 41.4839313258427,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.08828125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.1484375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108331.19920870892,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108331.19920870892,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001182,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2182325.83689895,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2085731.0276360807,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
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
          "id": "b639b30ee6fb2e1453a80c71a0b29c018c233980",
          "message": "Columnar query engine support conditionally executing pipeline stages (#1684)\n\npart of #1667 \n\nAdds a new `ConditionalDataExpression` to the transformation expression\nAST for applying transformation `PipelineStages` to some subset of rows\nthat match a condition. This is used to implement a\n`ConditionalPipelineStage`, which operates like `if/else if/else`\ncontrol flow.\n\nFor example, imagine we had a hypothetical syntax like:\n```kql\nlogs |\n  if (severity_text == \"ERROR\") {\n     set attributes[\"important\"] = \"very\" | set attributes[\"triggers_alarm\"] = true\n  } else if (severity_text == \"WARN) {\n     set attributes[\"important\"] = \"somewhat\"\n  } else {\n     set attributes[\"important\"] = \"no\"\n  }\n```\n\nThis could be modeled using our conditional expression like:\n```rs\n// this is pesudocode to illustrate what each field represents\nConditional {\n  branches: [\n     ConditionalBranch {\n       condition: \"severity_text == \\\"ERROR\\\"\",\n       expressions: [ \n         \"set attributes[\\\"important\\\"] = \\\"very\\\"\",\n         \"set attributes[\\\"triggers_alarm\\\"] = true\"  \n      ],\n     },\n     ConditionalBranch {\n       condition: \"severity_text == \\\"WARN\\\"\",\n       expressions: [\n        \"set attributes[\\\"important\\\"] = \\\"somewhat\\\"\n      ],\n     },\n  ],\n  default_branch: Some([\n    \"set attributes[\"important\"] = \\\"no\\\"\"\n  ])\n}\n```\n\nNote there is currently no parser support for a language syntax that\ncreates this variant of `DataExpression`. That will happen in a future\nPR\n\n---------\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2025-12-23T23:56:20Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/b639b30ee6fb2e1453a80c71a0b29c018c233980"
        },
        "date": 1766572324745,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 40.72432267248986,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 41.241605314878896,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 40.72432267248986,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 41.241605314878896,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 38.807291666666664,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 40.8515625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108331.50794742443,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108331.50794742443,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001011,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2190839.431948412,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2093607.3323782827,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 42.90966637333208,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 43.90163105365023,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 42.90966637333208,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 43.90163105365023,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 38.885677083333334,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 40.98828125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108331.36170255035,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108331.36170255035,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001092,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2398064.812820648,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 740353.4569303951,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
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
          "id": "b639b30ee6fb2e1453a80c71a0b29c018c233980",
          "message": "Columnar query engine support conditionally executing pipeline stages (#1684)\n\npart of #1667 \n\nAdds a new `ConditionalDataExpression` to the transformation expression\nAST for applying transformation `PipelineStages` to some subset of rows\nthat match a condition. This is used to implement a\n`ConditionalPipelineStage`, which operates like `if/else if/else`\ncontrol flow.\n\nFor example, imagine we had a hypothetical syntax like:\n```kql\nlogs |\n  if (severity_text == \"ERROR\") {\n     set attributes[\"important\"] = \"very\" | set attributes[\"triggers_alarm\"] = true\n  } else if (severity_text == \"WARN) {\n     set attributes[\"important\"] = \"somewhat\"\n  } else {\n     set attributes[\"important\"] = \"no\"\n  }\n```\n\nThis could be modeled using our conditional expression like:\n```rs\n// this is pesudocode to illustrate what each field represents\nConditional {\n  branches: [\n     ConditionalBranch {\n       condition: \"severity_text == \\\"ERROR\\\"\",\n       expressions: [ \n         \"set attributes[\\\"important\\\"] = \\\"very\\\"\",\n         \"set attributes[\\\"triggers_alarm\\\"] = true\"  \n      ],\n     },\n     ConditionalBranch {\n       condition: \"severity_text == \\\"WARN\\\"\",\n       expressions: [\n        \"set attributes[\\\"important\\\"] = \\\"somewhat\\\"\n      ],\n     },\n  ],\n  default_branch: Some([\n    \"set attributes[\"important\"] = \\\"no\\\"\"\n  ])\n}\n```\n\nNote there is currently no parser support for a language syntax that\ncreates this variant of `DataExpression`. That will happen in a future\nPR\n\n---------\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2025-12-23T23:56:20Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/b639b30ee6fb2e1453a80c71a0b29c018c233980"
        },
        "date": 1766658637646,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 40.6784822132728,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 41.646287663508204,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 40.6784822132728,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 41.646287663508204,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.586328125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.20703125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108332.03876547009,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108332.03876547009,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000717,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2177758.3416145057,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2081398.8779387835,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 43.236718909388124,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 44.27860370358959,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 43.236718909388124,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 44.27860370358959,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 38.66640625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 40.69140625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108331.6397486986,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108331.6397486986,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000938,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2401245.007481715,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 741235.7090427924,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
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
          "id": "b639b30ee6fb2e1453a80c71a0b29c018c233980",
          "message": "Columnar query engine support conditionally executing pipeline stages (#1684)\n\npart of #1667 \n\nAdds a new `ConditionalDataExpression` to the transformation expression\nAST for applying transformation `PipelineStages` to some subset of rows\nthat match a condition. This is used to implement a\n`ConditionalPipelineStage`, which operates like `if/else if/else`\ncontrol flow.\n\nFor example, imagine we had a hypothetical syntax like:\n```kql\nlogs |\n  if (severity_text == \"ERROR\") {\n     set attributes[\"important\"] = \"very\" | set attributes[\"triggers_alarm\"] = true\n  } else if (severity_text == \"WARN) {\n     set attributes[\"important\"] = \"somewhat\"\n  } else {\n     set attributes[\"important\"] = \"no\"\n  }\n```\n\nThis could be modeled using our conditional expression like:\n```rs\n// this is pesudocode to illustrate what each field represents\nConditional {\n  branches: [\n     ConditionalBranch {\n       condition: \"severity_text == \\\"ERROR\\\"\",\n       expressions: [ \n         \"set attributes[\\\"important\\\"] = \\\"very\\\"\",\n         \"set attributes[\\\"triggers_alarm\\\"] = true\"  \n      ],\n     },\n     ConditionalBranch {\n       condition: \"severity_text == \\\"WARN\\\"\",\n       expressions: [\n        \"set attributes[\\\"important\\\"] = \\\"somewhat\\\"\n      ],\n     },\n  ],\n  default_branch: Some([\n    \"set attributes[\"important\"] = \\\"no\\\"\"\n  ])\n}\n```\n\nNote there is currently no parser support for a language syntax that\ncreates this variant of `DataExpression`. That will happen in a future\nPR\n\n---------\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2025-12-23T23:56:20Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/b639b30ee6fb2e1453a80c71a0b29c018c233980"
        },
        "date": 1766745051744,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 42.75579111039812,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 44.1467605866377,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 42.75579111039812,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 44.1467605866377,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.17005208333333,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.56640625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108331.03310439708,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108331.03310439708,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001274,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2399464.817195683,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 733395.9289568961,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 40.622960199725995,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 41.63463119913353,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 40.622960199725995,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 41.63463119913353,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.47447916666667,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.25,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108331.67044219204,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108331.67044219204,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000921,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2187561.4210626995,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2092648.417489109,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
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
          "id": "8e81cc86d7a9574082e2c10d3bf4d2d02faa49b1",
          "message": "[query-engine] KQL function parsing + type conversion (#1668)\n\nRelates to #1479\n\n## Changes\n\n* Adds support for function definition and invocation in KQL parser\n* Implements automatic type conversion for functions in RecordSet engine",
          "timestamp": "2025-12-26T22:23:48Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/8e81cc86d7a9574082e2c10d3bf4d2d02faa49b1"
        },
        "date": 1766831486581,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 42.00562523695862,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 43.007816381304956,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 42.00562523695862,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 43.007816381304956,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.57630208333333,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.08203125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108331.61266621882,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108331.61266621882,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000953,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2186329.267317382,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2085359.649875532,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 10000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.1538461595773697,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 44.128306555840105,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 46.229933409073226,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 44.128306555840105,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 46.229933409073226,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.316015625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 40.9375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 6490000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108331.41406178087,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108164.75034783967,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001063,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2396931.939195925,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 739252.7538885226,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
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
          "id": "8e81cc86d7a9574082e2c10d3bf4d2d02faa49b1",
          "message": "[query-engine] KQL function parsing + type conversion (#1668)\n\nRelates to #1479\n\n## Changes\n\n* Adds support for function definition and invocation in KQL parser\n* Implements automatic type conversion for functions in RecordSet engine",
          "timestamp": "2025-12-26T22:23:48Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/8e81cc86d7a9574082e2c10d3bf4d2d02faa49b1"
        },
        "date": 1766917831479,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 41.65063770810929,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 42.479575106186516,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 41.65063770810929,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 42.479575106186516,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.98684895833333,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.2734375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108331.97918359355,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108331.97918359355,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00075,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2179282.892950597,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2081472.6603991024,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 7000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.10769230872392654,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 43.62141319507848,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 45.015240811415346,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 43.62141319507848,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 45.015240811415346,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.05247395833333,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.10546875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 6493000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108332.06043162325,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108215.39513577381,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000705,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2398201.7132267873,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 739526.619385753,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
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
          "id": "8e81cc86d7a9574082e2c10d3bf4d2d02faa49b1",
          "message": "[query-engine] KQL function parsing + type conversion (#1668)\n\nRelates to #1479\n\n## Changes\n\n* Adds support for function definition and invocation in KQL parser\n* Implements automatic type conversion for functions in RecordSet engine",
          "timestamp": "2025-12-26T22:23:48Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/8e81cc86d7a9574082e2c10d3bf4d2d02faa49b1"
        },
        "date": 1767004294819,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 42.13753253628483,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 44.65008254674703,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 42.13753253628483,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 44.65008254674703,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.225,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.29296875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108331.44656063907,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108331.44656063907,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001045,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2179928.9305292475,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2083491.422437363,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 9000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.13846154510974884,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 44.22285790355552,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 45.359707745090425,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 44.22285790355552,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 45.359707745090425,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 38.85169270833333,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 40.96875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 6491000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108331.78779982739,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108181.78993979686,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000856,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2383034.233306049,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 740375.2634607914,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
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
          "id": "66b4c7e30dca8c44340dace1056ed5a5887366ae",
          "message": "chore(deps): update dependency psutil to v7.2.1 (#1698)\n\nThis PR contains the following updates:\n\n| Package | Change |\n[Age](https://docs.renovatebot.com/merge-confidence/) |\n[Confidence](https://docs.renovatebot.com/merge-confidence/) |\n|---|---|---|---|\n| [psutil](https://redirect.github.com/giampaolo/psutil) | `==7.1.3` ->\n`==7.2.1` |\n![age](https://developer.mend.io/api/mc/badges/age/pypi/psutil/7.2.1?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/pypi/psutil/7.1.3/7.2.1?slim=true)\n|\n\n---\n\n### Release Notes\n\n<details>\n<summary>giampaolo/psutil (psutil)</summary>\n\n###\n[`v7.2.1`](https://redirect.github.com/giampaolo/psutil/blob/HEAD/HISTORY.rst#721)\n\n[Compare\nSource](https://redirect.github.com/giampaolo/psutil/compare/release-7.2.0...release-7.2.1)\n\n\\=====\n\n2025-12-29\n\n**Bug fixes**\n\n- 2699\\_, \\[FreeBSD], \\[NetBSD]: `heap_info()`\\_ does not detect small\nallocations\n(<= 1K). In order to fix that, we now flush internal jemalloc cache\nbefore\n  fetching the metrics.\n\n###\n[`v7.2.0`](https://redirect.github.com/giampaolo/psutil/blob/HEAD/HISTORY.rst#720)\n\n[Compare\nSource](https://redirect.github.com/giampaolo/psutil/compare/release-7.1.3...release-7.2.0)\n\n\\=====\n\n2025-12-23\n\n**Enhancements**\n\n- 1275\\_: new `heap_info()`\\_ and `heap_trim()`\\_ functions, providing\ndirect\n  access to the platform's native C heap allocator (glibc, mimalloc,\n  libmalloc). Useful to create tools to detect memory leaks.\n- 2403\\_, \\[Linux]: publish wheels for Linux musl.\n- 2680\\_: unit tests are no longer installed / part of the distribution.\nThey\n  now live under `tests/` instead of `psutil/tests`.\n\n**Bug fixes**\n\n- 2684\\_, \\[FreeBSD], \\[critical]: compilation fails on FreeBSD 14 due\nto missing\n  include.\n- 2691\\_, \\[Windows]: fix memory leak in `net_if_stats()`\\_ due to\nmissing\n  `Py_CLEAR`.\n\n**Compatibility notes**\n\n- 2680\\_: `import psutil.tests` no longer works (but it was never\ndocumented to\n  begin with).\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - \"before 8am on Monday\" (UTC),\nAutomerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0Mi41OS4wIiwidXBkYXRlZEluVmVyIjoiNDIuNTkuMCIsInRhcmdldEJyYW5jaCI6Im1haW4iLCJsYWJlbHMiOlsiZGVwZW5kZW5jaWVzIl19-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2025-12-29T19:06:30Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/66b4c7e30dca8c44340dace1056ed5a5887366ae"
        },
        "date": 1767090722365,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 7000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.10769230872392654,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 44.072250011338085,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 45.82540545327862,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 44.072250011338085,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 45.82540545327862,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 37.655078125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.15625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 6493000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108331.42128374768,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108214.75667621134,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001059,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2393383.2320309747,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 738325.1158436028,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 41.88195828435708,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 42.759794497223936,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 41.88195828435708,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 42.759794497223936,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.85546875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.30078125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108331.65960919237,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108331.65960919237,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000927,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2175824.268025174,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2077894.0958341644,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
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
          "id": "1af4d28d2119f57dae33f166f635ef16d69223a5",
          "message": "feat: Initial Condense processor implementation (#1695)\n\nRelated to #1435\n\nFixes #1693\n\nThis is a basic implementation of the `Condense` behavior from the above\nissue that works for `LogAttrs` payload types.\n\nThis iteration currently builds an entirely new `RecordBatch` during\nexecution. As mentioned in comments, once #1035 is completed, working\nin-place on the existing `RecordBatch` would be more efficient\nespecially with respect to persisted attributes.\n\nWith a debug pipeline configuration composed of:\n* `syslog_cef_receiver`\n* `attributes_processor` (doing various renames and deletes)\n* `condense_attributes_processor`\n\nSending the CEF message:\n> <134>Dec 29 17:28:13 securityhost\nCEF:0|Security|threatmanager|1.0|100|worm successfully\nstopped|10|src=10.0.0.1 dst=2.1.2.2 spt=1232 dpt=80 proto=tcp\nact=blocked vendorspecificext1=value1 vendorspecificext2=value2\n\nResults in the following LogRecord:\n```\nLogRecord #0:\n   -> ObservedTimestamp: 1767029293753398998\n   -> Timestamp: 1767029293000000000\n   -> SeverityText: INFO\n   -> SeverityNumber: 9\n   -> Body: <134>Dec 29 17:28:13 securityhost CEF:0|Security|threatmanager|1.0|100|worm successfully stopped|10|src=10.0.0.1 dst=2.1.2.2 spt=1232 dpt=80 proto=tcp act=blocked vendorspecificext1=value1 vendorspecificext2=value2\n   -> Attributes:\n      -> AdditionalExtensions: vendorspecificext1=value1|vendorspecificext2=value2\n      -> Computer: securityhost\n      -> DeviceVendor: Security\n      -> DeviceProduct: threatmanager\n      -> DeviceVersion: 1.0\n      -> DeviceEventClassId: 100\n      -> Activity: worm successfully stopped\n      -> LogSeverity: 10\n      -> SourceIP: 10.0.0.1\n      -> DestinationIP: 2.1.2.2\n      -> SourcePort: 1232\n      -> DestinationPort: 80\n      -> Protocol: tcp\n      -> DeviceAction: blocked\n   -> Trace ID:\n   -> Span ID:\n   -> Flags: 0 \n```",
          "timestamp": "2025-12-31T00:42:40Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/1af4d28d2119f57dae33f166f635ef16d69223a5"
        },
        "date": 1767177147558,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 11000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.16923075914382935,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 43.81735608569653,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 45.64191326107154,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 43.81735608569653,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 45.64191326107154,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.47083333333333,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 6489000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108330.27119766748,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108146.94304640988,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001696,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2414927.6477196408,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 740211.7552825377,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 41.37575035209213,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 42.74675601448717,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 41.37575035209213,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 42.74675601448717,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.427083333333336,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 40.96875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108331.78599432338,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108331.78599432338,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000857,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2196592.0072542867,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2082245.9091786684,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
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
          "id": "1af4d28d2119f57dae33f166f635ef16d69223a5",
          "message": "feat: Initial Condense processor implementation (#1695)\n\nRelated to #1435\n\nFixes #1693\n\nThis is a basic implementation of the `Condense` behavior from the above\nissue that works for `LogAttrs` payload types.\n\nThis iteration currently builds an entirely new `RecordBatch` during\nexecution. As mentioned in comments, once #1035 is completed, working\nin-place on the existing `RecordBatch` would be more efficient\nespecially with respect to persisted attributes.\n\nWith a debug pipeline configuration composed of:\n* `syslog_cef_receiver`\n* `attributes_processor` (doing various renames and deletes)\n* `condense_attributes_processor`\n\nSending the CEF message:\n> <134>Dec 29 17:28:13 securityhost\nCEF:0|Security|threatmanager|1.0|100|worm successfully\nstopped|10|src=10.0.0.1 dst=2.1.2.2 spt=1232 dpt=80 proto=tcp\nact=blocked vendorspecificext1=value1 vendorspecificext2=value2\n\nResults in the following LogRecord:\n```\nLogRecord #0:\n   -> ObservedTimestamp: 1767029293753398998\n   -> Timestamp: 1767029293000000000\n   -> SeverityText: INFO\n   -> SeverityNumber: 9\n   -> Body: <134>Dec 29 17:28:13 securityhost CEF:0|Security|threatmanager|1.0|100|worm successfully stopped|10|src=10.0.0.1 dst=2.1.2.2 spt=1232 dpt=80 proto=tcp act=blocked vendorspecificext1=value1 vendorspecificext2=value2\n   -> Attributes:\n      -> AdditionalExtensions: vendorspecificext1=value1|vendorspecificext2=value2\n      -> Computer: securityhost\n      -> DeviceVendor: Security\n      -> DeviceProduct: threatmanager\n      -> DeviceVersion: 1.0\n      -> DeviceEventClassId: 100\n      -> Activity: worm successfully stopped\n      -> LogSeverity: 10\n      -> SourceIP: 10.0.0.1\n      -> DestinationIP: 2.1.2.2\n      -> SourcePort: 1232\n      -> DestinationPort: 80\n      -> Protocol: tcp\n      -> DeviceAction: blocked\n   -> Trace ID:\n   -> Span ID:\n   -> Flags: 0 \n```",
          "timestamp": "2025-12-31T00:42:40Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/1af4d28d2119f57dae33f166f635ef16d69223a5"
        },
        "date": 1767263501982,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 41.449299185553095,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 42.51634476146148,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 41.449299185553095,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 42.51634476146148,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 40.10364583333333,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.0390625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108331.6054442265,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108331.6054442265,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000957,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2199216.1561668175,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2080582.8171942094,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 9000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.13846154510974884,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 43.933274003031094,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 45.38195017652524,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 43.933274003031094,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 45.38195017652524,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 40.36731770833333,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.5625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 6491000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108330.10329075354,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108180.10776312019,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001789,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2409188.858564707,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 739401.9635615748,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
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
          "id": "1af4d28d2119f57dae33f166f635ef16d69223a5",
          "message": "feat: Initial Condense processor implementation (#1695)\n\nRelated to #1435\n\nFixes #1693\n\nThis is a basic implementation of the `Condense` behavior from the above\nissue that works for `LogAttrs` payload types.\n\nThis iteration currently builds an entirely new `RecordBatch` during\nexecution. As mentioned in comments, once #1035 is completed, working\nin-place on the existing `RecordBatch` would be more efficient\nespecially with respect to persisted attributes.\n\nWith a debug pipeline configuration composed of:\n* `syslog_cef_receiver`\n* `attributes_processor` (doing various renames and deletes)\n* `condense_attributes_processor`\n\nSending the CEF message:\n> <134>Dec 29 17:28:13 securityhost\nCEF:0|Security|threatmanager|1.0|100|worm successfully\nstopped|10|src=10.0.0.1 dst=2.1.2.2 spt=1232 dpt=80 proto=tcp\nact=blocked vendorspecificext1=value1 vendorspecificext2=value2\n\nResults in the following LogRecord:\n```\nLogRecord #0:\n   -> ObservedTimestamp: 1767029293753398998\n   -> Timestamp: 1767029293000000000\n   -> SeverityText: INFO\n   -> SeverityNumber: 9\n   -> Body: <134>Dec 29 17:28:13 securityhost CEF:0|Security|threatmanager|1.0|100|worm successfully stopped|10|src=10.0.0.1 dst=2.1.2.2 spt=1232 dpt=80 proto=tcp act=blocked vendorspecificext1=value1 vendorspecificext2=value2\n   -> Attributes:\n      -> AdditionalExtensions: vendorspecificext1=value1|vendorspecificext2=value2\n      -> Computer: securityhost\n      -> DeviceVendor: Security\n      -> DeviceProduct: threatmanager\n      -> DeviceVersion: 1.0\n      -> DeviceEventClassId: 100\n      -> Activity: worm successfully stopped\n      -> LogSeverity: 10\n      -> SourceIP: 10.0.0.1\n      -> DestinationIP: 2.1.2.2\n      -> SourcePort: 1232\n      -> DestinationPort: 80\n      -> Protocol: tcp\n      -> DeviceAction: blocked\n   -> Trace ID:\n   -> Span ID:\n   -> Flags: 0 \n```",
          "timestamp": "2025-12-31T00:42:40Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/1af4d28d2119f57dae33f166f635ef16d69223a5"
        },
        "date": 1767349851850,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 41.53246280889517,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 42.55453191851909,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 41.53246280889517,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 42.55453191851909,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 40.14505208333333,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.421875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108330.64131689661,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108330.64131689661,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001491,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2197642.3912256015,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2083159.1538954491,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 11000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.16923075914382935,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 43.90596260454493,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 45.68785334471255,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 43.90596260454493,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 45.68785334471255,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.840755208333334,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.4296875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 6489000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108331.9232127995,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108148.592265824,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000781,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2412741.589293017,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 739368.8260407001,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
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
          "id": "1af4d28d2119f57dae33f166f635ef16d69223a5",
          "message": "feat: Initial Condense processor implementation (#1695)\n\nRelated to #1435\n\nFixes #1693\n\nThis is a basic implementation of the `Condense` behavior from the above\nissue that works for `LogAttrs` payload types.\n\nThis iteration currently builds an entirely new `RecordBatch` during\nexecution. As mentioned in comments, once #1035 is completed, working\nin-place on the existing `RecordBatch` would be more efficient\nespecially with respect to persisted attributes.\n\nWith a debug pipeline configuration composed of:\n* `syslog_cef_receiver`\n* `attributes_processor` (doing various renames and deletes)\n* `condense_attributes_processor`\n\nSending the CEF message:\n> <134>Dec 29 17:28:13 securityhost\nCEF:0|Security|threatmanager|1.0|100|worm successfully\nstopped|10|src=10.0.0.1 dst=2.1.2.2 spt=1232 dpt=80 proto=tcp\nact=blocked vendorspecificext1=value1 vendorspecificext2=value2\n\nResults in the following LogRecord:\n```\nLogRecord #0:\n   -> ObservedTimestamp: 1767029293753398998\n   -> Timestamp: 1767029293000000000\n   -> SeverityText: INFO\n   -> SeverityNumber: 9\n   -> Body: <134>Dec 29 17:28:13 securityhost CEF:0|Security|threatmanager|1.0|100|worm successfully stopped|10|src=10.0.0.1 dst=2.1.2.2 spt=1232 dpt=80 proto=tcp act=blocked vendorspecificext1=value1 vendorspecificext2=value2\n   -> Attributes:\n      -> AdditionalExtensions: vendorspecificext1=value1|vendorspecificext2=value2\n      -> Computer: securityhost\n      -> DeviceVendor: Security\n      -> DeviceProduct: threatmanager\n      -> DeviceVersion: 1.0\n      -> DeviceEventClassId: 100\n      -> Activity: worm successfully stopped\n      -> LogSeverity: 10\n      -> SourceIP: 10.0.0.1\n      -> DestinationIP: 2.1.2.2\n      -> SourcePort: 1232\n      -> DestinationPort: 80\n      -> Protocol: tcp\n      -> DeviceAction: blocked\n   -> Trace ID:\n   -> Span ID:\n   -> Flags: 0 \n```",
          "timestamp": "2025-12-31T00:42:40Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/1af4d28d2119f57dae33f166f635ef16d69223a5"
        },
        "date": 1767436212226,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 11000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.16923075914382935,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 44.60490809867343,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 45.786971994429834,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 44.60490809867343,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 45.786971994429834,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.553385416666664,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 6489000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108331.81668789971,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108148.4859211971,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00084,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2416253.447639598,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 739738.3639487519,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 41.93892416524128,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 43.1944955648277,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 41.93892416524128,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 43.1944955648277,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.82630208333333,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.2890625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108331.60363872857,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108331.60363872857,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000958,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2195000.0960878725,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2081392.4393584274,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Andres Borja",
            "username": "andborja",
            "email": "76450334+andborja@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "2584e4b429b131115964742db4115782e6b19781",
          "message": "feat: Add internal telemetry prometheus exporter (#1691)\n\nAdd internal telemetry configurable prometheus exporter.",
          "timestamp": "2026-01-03T22:00:15Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/2584e4b429b131115964742db4115782e6b19781"
        },
        "date": 1767522707499,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 10000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.1538461595773697,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 44.28952245048356,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 45.5339111510457,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 44.28952245048356,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 45.5339111510457,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 38.723828125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 39.27734375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 6490000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108328.27079214499,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108161.61191400322,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002804,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2406548.877750333,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 738736.9674902713,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 42.52576915239426,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 43.27480853745173,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 42.52576915239426,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 43.27480853745173,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.8828125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.09765625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108331.53502985183,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108331.53502985183,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000996,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2187150.150277574,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2072599.4144781905,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Andres Borja",
            "username": "andborja",
            "email": "76450334+andborja@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "2584e4b429b131115964742db4115782e6b19781",
          "message": "feat: Add internal telemetry prometheus exporter (#1691)\n\nAdd internal telemetry configurable prometheus exporter.",
          "timestamp": "2026-01-03T22:00:15Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/2584e4b429b131115964742db4115782e6b19781"
        },
        "date": 1767609166780,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 42.50213350052752,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 43.508492440604755,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 42.50213350052752,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 43.508492440604755,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.65872395833333,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.41015625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108331.91057424112,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108331.91057424112,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000788,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2190623.4055911014,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2069306.0114550807,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 12000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.1846153885126114,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 44.11168713661179,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 45.976752746243086,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 44.11168713661179,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 45.976752746243086,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 38.75833333333333,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 39.46875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 6488000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108328.39897476003,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108128.4080843451,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002733,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2408105.284222635,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 739126.4685197399,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
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
          "id": "4bf9bf85b4d0c2e7992e18e3dedda40427aa2485",
          "message": "[batch_processor] Support bytes-based batching via new `format = [otap|otlp|preserve]` (#1633)\n\nFixes #1570.\n\nAdds dual format configuration to batch processor, with separate\n`FormatConfig` structs for each payload format.\nThis supports forcing payload into one or the other format, or allowing\nboth to be preserved.\n\nThe new bytes-based batching routines operate by scanning through\ntop-level fields. Unlike the items-based batching mode, this may produce\nbatches that are less than the limit; like that mode, it can also\nproduce outputs greater than the limit.",
          "timestamp": "2026-01-06T01:10:06Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/4bf9bf85b4d0c2e7992e18e3dedda40427aa2485"
        },
        "date": 1767695611976,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 41.79328863965513,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 42.58070199444273,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.896875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.28515625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 108331.98098910399,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108331.98098910399,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000749,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2212612.5092413654,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2099640.131018896,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.1846153885126114,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 45.80250899374855,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 47.36515389162562,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 40.084895833333334,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.36328125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 108329.50027451529,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108129.50735093157,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002123,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2419285.6853555194,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 740031.3768771103,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Andres Borja",
            "username": "andborja",
            "email": "76450334+andborja@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "4d58ac8f2141e9fcc0baaa73cc7cdbeac38993eb",
          "message": "feat: Add 'tls' option to internal telemetry OTLP configuration (#1724)\n\nAdd 'tls' option to internal telemetry OTLP configuration with ca file.",
          "timestamp": "2026-01-06T21:31:56Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/4d58ac8f2141e9fcc0baaa73cc7cdbeac38993eb"
        },
        "date": 1767782005008,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.1846153885126114,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 44.82264509743398,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 46.222621439405664,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 38.73802083333333,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.44921875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 108330.26939221402,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108130.27504872071,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001697,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2421242.3421489955,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 740359.7339499405,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 40.89807650545097,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 41.772265975948194,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.94544270833333,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.14453125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 108330.46979791501,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108330.46979791501,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001586,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2214167.815181508,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2099968.338371989,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
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
          "id": "9b0aa84cfc1ef5df2f4434a3290bab3abbd7299c",
          "message": "[kql_processor] Experimental KQL recordset processor (#1730)\n\nRelates to #1642\n\n## Changes\n\n* Adds a processor which executes KQL query using the RecordSet engine\n(OTLP-bytes form)\n\n## Details\n\n@drewrelmas took @jmacd's original work and added tests + config. I\ncleaned it up a bit and improved the bridge API to allow the processor\nto own the pipeline memory instead of storing it in a static. The static\npath is in place for callers needing to invoke things using FFI (from\nnon-Rust platforms).\n\n---------\n\nCo-authored-by: Drew Relmas <drewrelmas@microsoft.com>",
          "timestamp": "2026-01-08T00:49:19Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/9b0aa84cfc1ef5df2f4434a3290bab3abbd7299c"
        },
        "date": 1767868440231,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.03076923079788685,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 44.258999462839796,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 45.66100954000773,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 37.075,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 39.55078125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 108331.59280574224,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108298.26000795586,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000964,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2417266.1604792415,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 740276.9204658442,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 41.065406193377726,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 43.11564239358421,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 40.325520833333336,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.34765625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 108331.76974479001,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108331.76974479001,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000866,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2203890.653341615,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2086642.1751237037,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
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
          "id": "7cffafe2cb2c3ac605852d8d87ba77b4a41b716c",
          "message": "Internal Telemetry Guidelines (#1727)\n\nThis PR defines a set of guidelines for our internal telemetry and for\ndescribing how we can establish a telemetry by design process.\n\nOnce this PR is merged, I will follow up with a series of PRs to align\nthe existing instrumentation with these recommendations.\n\n---------\n\nCo-authored-by: Cijo Thomas <cithomas@microsoft.com>",
          "timestamp": "2026-01-08T22:23:58Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/7cffafe2cb2c3ac605852d8d87ba77b4a41b716c"
        },
        "date": 1767954751801,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.13846154510974884,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 44.299795692753314,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 45.78135411511234,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.836979166666666,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.7734375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 108329.98593676789,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108179.99057162467,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001854,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2421571.685592915,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 740301.325944078,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 41.34688825567662,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 42.01504492383824,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 40.148828125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.359375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 108329.47138767836,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108329.47138767836,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002139,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2204743.191563653,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2088934.9576982793,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
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
          "id": "002f4368ddd47cc05a69bd93e39b7f27850d9bc7",
          "message": "Internal logging code path: Raw logger support (#1735)\n\nImplements new internal logging configuration option.\n\nChanges the default logging configuration to use internal logging at\nlevel INFO. Previously, default logging was disabled.\n\nImplements a lightweight Tokio tracing layer to construct\npartially-encoded OTLP bytes from the Event, forming a struct that can\nbe passed through a channel to a global subscriber.\n\nAs the first step, implements \"raw logging\" directly to the console\nusing simple write! macros and the view object for LogRecord to\ninterpret the partial encoding and print it. The raw logging limits\nconsole message size to 4KiB.\n\nAdds a new `configs/internal-telemetry.yaml` to demonstrate this\nconfiguration.\n\nAdds benchmarks showing good performance, in the 50-200ns range to\nencode or encode/format:\n\n```\nencode/0_attrs/100_events\n                        time:   [5.5326 µs 5.5691 µs 5.6054 µs]\n                        change: [−7.3098% −4.0342% −1.9226%] (p = 0.00 < 0.05)\n                        Performance has improved.\nFound 1 outliers among 100 measurements (1.00%)\n  1 (1.00%) high mild\nencode/3_attrs/100_events\n                        time:   [8.5902 µs 8.6810 µs 8.7775 µs]\n                        change: [−5.7968% −3.2559% −1.1958%] (p = 0.00 < 0.05)\n                        Performance has improved.\nFound 7 outliers among 100 measurements (7.00%)\n  2 (2.00%) low mild\n  2 (2.00%) high mild\n  3 (3.00%) high severe\nencode/10_attrs/100_events\n                        time:   [19.583 µs 19.764 µs 19.944 µs]\n                        change: [−1.5682% +0.0078% +1.3193%] (p = 0.99 > 0.05)\n                        No change in performance detected.\nFound 3 outliers among 100 measurements (3.00%)\n  3 (3.00%) high mild\nencode/0_attrs/1000_events\n                        time:   [53.424 µs 53.874 µs 54.289 µs]\n                        change: [−2.8602% −1.8582% −0.9413%] (p = 0.00 < 0.05)\n                        Change within noise threshold.\nFound 2 outliers among 100 measurements (2.00%)\n  1 (1.00%) low mild\n  1 (1.00%) high severe\nencode/3_attrs/1000_events\n                        time:   [84.768 µs 85.161 µs 85.562 µs]\n                        change: [−3.3406% −2.4035% −1.5473%] (p = 0.00 < 0.05)\n                        Performance has improved.\nFound 5 outliers among 100 measurements (5.00%)\n  1 (1.00%) low mild\n  4 (4.00%) high mild\nencode/10_attrs/1000_events\n                        time:   [193.04 µs 194.07 µs 195.13 µs]\n                        change: [−1.8940% −0.1358% +1.7994%] (p = 0.89 > 0.05)\n                        No change in performance detected.\nFound 7 outliers among 100 measurements (7.00%)\n  1 (1.00%) low severe\n  1 (1.00%) low mild\n  2 (2.00%) high mild\n  3 (3.00%) high severe\n\nformat/0_attrs/100_events\n                        time:   [26.281 µs 26.451 µs 26.633 µs]\n                        change: [−16.944% −14.312% −10.992%] (p = 0.00 < 0.05)\n                        Performance has improved.\nFound 6 outliers among 100 measurements (6.00%)\n  1 (1.00%) low mild\n  1 (1.00%) high mild\n  4 (4.00%) high severe\nformat/3_attrs/100_events\n                        time:   [38.813 µs 39.180 µs 39.603 µs]\n                        change: [−8.0880% −6.7812% −5.5109%] (p = 0.00 < 0.05)\n                        Performance has improved.\nFound 8 outliers among 100 measurements (8.00%)\n  1 (1.00%) low severe\n  1 (1.00%) low mild\n  4 (4.00%) high mild\n  2 (2.00%) high severe\nformat/10_attrs/100_events\n                        time:   [70.655 µs 71.176 µs 71.752 µs]\n                        change: [−4.8840% −3.9457% −3.0096%] (p = 0.00 < 0.05)\n                        Performance has improved.\nFound 4 outliers among 100 measurements (4.00%)\n  4 (4.00%) high mild\nformat/0_attrs/1000_events\n                        time:   [295.80 µs 310.56 µs 325.75 µs]\n                        change: [−3.2629% −0.5673% +2.4337%] (p = 0.71 > 0.05)\n                        No change in performance detected.\nFound 10 outliers among 100 measurements (10.00%)\n  3 (3.00%) high mild\n  7 (7.00%) high severe\nformat/3_attrs/1000_events\n                        time:   [422.93 µs 430.42 µs 439.21 µs]\n                        change: [−1.3953% +0.8886% +3.3330%] (p = 0.46 > 0.05)\n                        No change in performance detected.\nFound 5 outliers among 100 measurements (5.00%)\n  5 (5.00%) high mild\nformat/10_attrs/1000_events\n                        time:   [720.96 µs 725.68 µs 730.81 µs]\n                        change: [−15.540% −13.383% −11.371%] (p = 0.00 < 0.05)\n                        Performance has improved.\nFound 9 outliers among 100 measurements (9.00%)\n  1 (1.00%) low mild\n  5 (5.00%) high mild\n  3 (3.00%) high severe\n\nencode_and_format/0_attrs/100_events\n                        time:   [32.698 µs 32.914 µs 33.147 µs]\n                        change: [−9.4066% −7.8944% −6.3427%] (p = 0.00 < 0.05)\n                        Performance has improved.\nFound 8 outliers among 100 measurements (8.00%)\n  2 (2.00%) low mild\n  3 (3.00%) high mild\n  3 (3.00%) high severe\nencode_and_format/3_attrs/100_events\n                        time:   [48.927 µs 49.498 µs 50.133 µs]\n                        change: [−7.2473% −5.1069% −2.7211%] (p = 0.00 < 0.05)\n                        Performance has improved.\nFound 10 outliers among 100 measurements (10.00%)\n  3 (3.00%) high mild\n  7 (7.00%) high severe\nencode_and_format/10_attrs/100_events\n                        time:   [95.328 µs 96.088 µs 96.970 µs]\n                        change: [−6.3169% −4.9414% −3.6501%] (p = 0.00 < 0.05)\n                        Performance has improved.\nFound 5 outliers among 100 measurements (5.00%)\n  4 (4.00%) high mild\n  1 (1.00%) high severe\nencode_and_format/0_attrs/1000_events\n                        time:   [326.65 µs 328.86 µs 331.27 µs]\n                        change: [−41.188% −39.915% −38.764%] (p = 0.00 < 0.05)\n                        Performance has improved.\nFound 7 outliers among 100 measurements (7.00%)\n  6 (6.00%) high mild\n  1 (1.00%) high severe\nencode_and_format/3_attrs/1000_events\n                        time:   [500.59 µs 504.82 µs 509.33 µs]\n                        change: [−50.787% −48.877% −47.483%] (p = 0.00 < 0.05)\n                        Performance has improved.\nFound 4 outliers among 100 measurements (4.00%)\n  3 (3.00%) high mild\n  1 (1.00%) high severe\nencode_and_format/10_attrs/1000_events\n                        time:   [944.34 µs 951.79 µs 960.38 µs]\n                        change: [−55.389% −54.741% −54.065%] (p = 0.00 < 0.05)\n                        Performance has improved.\nFound 4 outliers among 100 measurements (4.00%)\n  3 (3.00%) high mild\n  1 (1.00%) high severe\n```\n\n---------\n\nCo-authored-by: Laurent Quérel <laurent.querel@gmail.com>",
          "timestamp": "2026-01-09T23:01:40Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/002f4368ddd47cc05a69bd93e39b7f27850d9bc7"
        },
        "date": 1768041105290,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.20000000298023224,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 44.624713721048956,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 45.81777617034408,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 38.74609375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 40.93359375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 108331.71919071738,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108115.05575233595,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000894,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2414315.846259934,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 738928.827426426,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 41.82770630171336,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 42.67044165739023,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 40.012109375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.2578125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 108330.28383584335,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108330.28383584335,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001689,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2201949.946300161,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2085990.1282317839,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
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
          "id": "002f4368ddd47cc05a69bd93e39b7f27850d9bc7",
          "message": "Internal logging code path: Raw logger support (#1735)\n\nImplements new internal logging configuration option.\n\nChanges the default logging configuration to use internal logging at\nlevel INFO. Previously, default logging was disabled.\n\nImplements a lightweight Tokio tracing layer to construct\npartially-encoded OTLP bytes from the Event, forming a struct that can\nbe passed through a channel to a global subscriber.\n\nAs the first step, implements \"raw logging\" directly to the console\nusing simple write! macros and the view object for LogRecord to\ninterpret the partial encoding and print it. The raw logging limits\nconsole message size to 4KiB.\n\nAdds a new `configs/internal-telemetry.yaml` to demonstrate this\nconfiguration.\n\nAdds benchmarks showing good performance, in the 50-200ns range to\nencode or encode/format:\n\n```\nencode/0_attrs/100_events\n                        time:   [5.5326 µs 5.5691 µs 5.6054 µs]\n                        change: [−7.3098% −4.0342% −1.9226%] (p = 0.00 < 0.05)\n                        Performance has improved.\nFound 1 outliers among 100 measurements (1.00%)\n  1 (1.00%) high mild\nencode/3_attrs/100_events\n                        time:   [8.5902 µs 8.6810 µs 8.7775 µs]\n                        change: [−5.7968% −3.2559% −1.1958%] (p = 0.00 < 0.05)\n                        Performance has improved.\nFound 7 outliers among 100 measurements (7.00%)\n  2 (2.00%) low mild\n  2 (2.00%) high mild\n  3 (3.00%) high severe\nencode/10_attrs/100_events\n                        time:   [19.583 µs 19.764 µs 19.944 µs]\n                        change: [−1.5682% +0.0078% +1.3193%] (p = 0.99 > 0.05)\n                        No change in performance detected.\nFound 3 outliers among 100 measurements (3.00%)\n  3 (3.00%) high mild\nencode/0_attrs/1000_events\n                        time:   [53.424 µs 53.874 µs 54.289 µs]\n                        change: [−2.8602% −1.8582% −0.9413%] (p = 0.00 < 0.05)\n                        Change within noise threshold.\nFound 2 outliers among 100 measurements (2.00%)\n  1 (1.00%) low mild\n  1 (1.00%) high severe\nencode/3_attrs/1000_events\n                        time:   [84.768 µs 85.161 µs 85.562 µs]\n                        change: [−3.3406% −2.4035% −1.5473%] (p = 0.00 < 0.05)\n                        Performance has improved.\nFound 5 outliers among 100 measurements (5.00%)\n  1 (1.00%) low mild\n  4 (4.00%) high mild\nencode/10_attrs/1000_events\n                        time:   [193.04 µs 194.07 µs 195.13 µs]\n                        change: [−1.8940% −0.1358% +1.7994%] (p = 0.89 > 0.05)\n                        No change in performance detected.\nFound 7 outliers among 100 measurements (7.00%)\n  1 (1.00%) low severe\n  1 (1.00%) low mild\n  2 (2.00%) high mild\n  3 (3.00%) high severe\n\nformat/0_attrs/100_events\n                        time:   [26.281 µs 26.451 µs 26.633 µs]\n                        change: [−16.944% −14.312% −10.992%] (p = 0.00 < 0.05)\n                        Performance has improved.\nFound 6 outliers among 100 measurements (6.00%)\n  1 (1.00%) low mild\n  1 (1.00%) high mild\n  4 (4.00%) high severe\nformat/3_attrs/100_events\n                        time:   [38.813 µs 39.180 µs 39.603 µs]\n                        change: [−8.0880% −6.7812% −5.5109%] (p = 0.00 < 0.05)\n                        Performance has improved.\nFound 8 outliers among 100 measurements (8.00%)\n  1 (1.00%) low severe\n  1 (1.00%) low mild\n  4 (4.00%) high mild\n  2 (2.00%) high severe\nformat/10_attrs/100_events\n                        time:   [70.655 µs 71.176 µs 71.752 µs]\n                        change: [−4.8840% −3.9457% −3.0096%] (p = 0.00 < 0.05)\n                        Performance has improved.\nFound 4 outliers among 100 measurements (4.00%)\n  4 (4.00%) high mild\nformat/0_attrs/1000_events\n                        time:   [295.80 µs 310.56 µs 325.75 µs]\n                        change: [−3.2629% −0.5673% +2.4337%] (p = 0.71 > 0.05)\n                        No change in performance detected.\nFound 10 outliers among 100 measurements (10.00%)\n  3 (3.00%) high mild\n  7 (7.00%) high severe\nformat/3_attrs/1000_events\n                        time:   [422.93 µs 430.42 µs 439.21 µs]\n                        change: [−1.3953% +0.8886% +3.3330%] (p = 0.46 > 0.05)\n                        No change in performance detected.\nFound 5 outliers among 100 measurements (5.00%)\n  5 (5.00%) high mild\nformat/10_attrs/1000_events\n                        time:   [720.96 µs 725.68 µs 730.81 µs]\n                        change: [−15.540% −13.383% −11.371%] (p = 0.00 < 0.05)\n                        Performance has improved.\nFound 9 outliers among 100 measurements (9.00%)\n  1 (1.00%) low mild\n  5 (5.00%) high mild\n  3 (3.00%) high severe\n\nencode_and_format/0_attrs/100_events\n                        time:   [32.698 µs 32.914 µs 33.147 µs]\n                        change: [−9.4066% −7.8944% −6.3427%] (p = 0.00 < 0.05)\n                        Performance has improved.\nFound 8 outliers among 100 measurements (8.00%)\n  2 (2.00%) low mild\n  3 (3.00%) high mild\n  3 (3.00%) high severe\nencode_and_format/3_attrs/100_events\n                        time:   [48.927 µs 49.498 µs 50.133 µs]\n                        change: [−7.2473% −5.1069% −2.7211%] (p = 0.00 < 0.05)\n                        Performance has improved.\nFound 10 outliers among 100 measurements (10.00%)\n  3 (3.00%) high mild\n  7 (7.00%) high severe\nencode_and_format/10_attrs/100_events\n                        time:   [95.328 µs 96.088 µs 96.970 µs]\n                        change: [−6.3169% −4.9414% −3.6501%] (p = 0.00 < 0.05)\n                        Performance has improved.\nFound 5 outliers among 100 measurements (5.00%)\n  4 (4.00%) high mild\n  1 (1.00%) high severe\nencode_and_format/0_attrs/1000_events\n                        time:   [326.65 µs 328.86 µs 331.27 µs]\n                        change: [−41.188% −39.915% −38.764%] (p = 0.00 < 0.05)\n                        Performance has improved.\nFound 7 outliers among 100 measurements (7.00%)\n  6 (6.00%) high mild\n  1 (1.00%) high severe\nencode_and_format/3_attrs/1000_events\n                        time:   [500.59 µs 504.82 µs 509.33 µs]\n                        change: [−50.787% −48.877% −47.483%] (p = 0.00 < 0.05)\n                        Performance has improved.\nFound 4 outliers among 100 measurements (4.00%)\n  3 (3.00%) high mild\n  1 (1.00%) high severe\nencode_and_format/10_attrs/1000_events\n                        time:   [944.34 µs 951.79 µs 960.38 µs]\n                        change: [−55.389% −54.741% −54.065%] (p = 0.00 < 0.05)\n                        Performance has improved.\nFound 4 outliers among 100 measurements (4.00%)\n  3 (3.00%) high mild\n  1 (1.00%) high severe\n```\n\n---------\n\nCo-authored-by: Laurent Quérel <laurent.querel@gmail.com>",
          "timestamp": "2026-01-09T23:01:40Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/002f4368ddd47cc05a69bd93e39b7f27850d9bc7"
        },
        "date": 1768127448591,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.2153846174478531,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 45.0353027509102,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 46.02536559548651,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.032291666666666,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.3125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 108330.17009236664,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108096.84357216768,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001752,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2417610.318158121,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 740453.7235032684,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 41.38981219862343,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 42.00376435290501,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 40.50182291666667,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.19140625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 108330.26758676063,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108330.26758676063,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001698,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2204856.3315464314,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2088399.6961409093,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
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
          "id": "002f4368ddd47cc05a69bd93e39b7f27850d9bc7",
          "message": "Internal logging code path: Raw logger support (#1735)\n\nImplements new internal logging configuration option.\n\nChanges the default logging configuration to use internal logging at\nlevel INFO. Previously, default logging was disabled.\n\nImplements a lightweight Tokio tracing layer to construct\npartially-encoded OTLP bytes from the Event, forming a struct that can\nbe passed through a channel to a global subscriber.\n\nAs the first step, implements \"raw logging\" directly to the console\nusing simple write! macros and the view object for LogRecord to\ninterpret the partial encoding and print it. The raw logging limits\nconsole message size to 4KiB.\n\nAdds a new `configs/internal-telemetry.yaml` to demonstrate this\nconfiguration.\n\nAdds benchmarks showing good performance, in the 50-200ns range to\nencode or encode/format:\n\n```\nencode/0_attrs/100_events\n                        time:   [5.5326 µs 5.5691 µs 5.6054 µs]\n                        change: [−7.3098% −4.0342% −1.9226%] (p = 0.00 < 0.05)\n                        Performance has improved.\nFound 1 outliers among 100 measurements (1.00%)\n  1 (1.00%) high mild\nencode/3_attrs/100_events\n                        time:   [8.5902 µs 8.6810 µs 8.7775 µs]\n                        change: [−5.7968% −3.2559% −1.1958%] (p = 0.00 < 0.05)\n                        Performance has improved.\nFound 7 outliers among 100 measurements (7.00%)\n  2 (2.00%) low mild\n  2 (2.00%) high mild\n  3 (3.00%) high severe\nencode/10_attrs/100_events\n                        time:   [19.583 µs 19.764 µs 19.944 µs]\n                        change: [−1.5682% +0.0078% +1.3193%] (p = 0.99 > 0.05)\n                        No change in performance detected.\nFound 3 outliers among 100 measurements (3.00%)\n  3 (3.00%) high mild\nencode/0_attrs/1000_events\n                        time:   [53.424 µs 53.874 µs 54.289 µs]\n                        change: [−2.8602% −1.8582% −0.9413%] (p = 0.00 < 0.05)\n                        Change within noise threshold.\nFound 2 outliers among 100 measurements (2.00%)\n  1 (1.00%) low mild\n  1 (1.00%) high severe\nencode/3_attrs/1000_events\n                        time:   [84.768 µs 85.161 µs 85.562 µs]\n                        change: [−3.3406% −2.4035% −1.5473%] (p = 0.00 < 0.05)\n                        Performance has improved.\nFound 5 outliers among 100 measurements (5.00%)\n  1 (1.00%) low mild\n  4 (4.00%) high mild\nencode/10_attrs/1000_events\n                        time:   [193.04 µs 194.07 µs 195.13 µs]\n                        change: [−1.8940% −0.1358% +1.7994%] (p = 0.89 > 0.05)\n                        No change in performance detected.\nFound 7 outliers among 100 measurements (7.00%)\n  1 (1.00%) low severe\n  1 (1.00%) low mild\n  2 (2.00%) high mild\n  3 (3.00%) high severe\n\nformat/0_attrs/100_events\n                        time:   [26.281 µs 26.451 µs 26.633 µs]\n                        change: [−16.944% −14.312% −10.992%] (p = 0.00 < 0.05)\n                        Performance has improved.\nFound 6 outliers among 100 measurements (6.00%)\n  1 (1.00%) low mild\n  1 (1.00%) high mild\n  4 (4.00%) high severe\nformat/3_attrs/100_events\n                        time:   [38.813 µs 39.180 µs 39.603 µs]\n                        change: [−8.0880% −6.7812% −5.5109%] (p = 0.00 < 0.05)\n                        Performance has improved.\nFound 8 outliers among 100 measurements (8.00%)\n  1 (1.00%) low severe\n  1 (1.00%) low mild\n  4 (4.00%) high mild\n  2 (2.00%) high severe\nformat/10_attrs/100_events\n                        time:   [70.655 µs 71.176 µs 71.752 µs]\n                        change: [−4.8840% −3.9457% −3.0096%] (p = 0.00 < 0.05)\n                        Performance has improved.\nFound 4 outliers among 100 measurements (4.00%)\n  4 (4.00%) high mild\nformat/0_attrs/1000_events\n                        time:   [295.80 µs 310.56 µs 325.75 µs]\n                        change: [−3.2629% −0.5673% +2.4337%] (p = 0.71 > 0.05)\n                        No change in performance detected.\nFound 10 outliers among 100 measurements (10.00%)\n  3 (3.00%) high mild\n  7 (7.00%) high severe\nformat/3_attrs/1000_events\n                        time:   [422.93 µs 430.42 µs 439.21 µs]\n                        change: [−1.3953% +0.8886% +3.3330%] (p = 0.46 > 0.05)\n                        No change in performance detected.\nFound 5 outliers among 100 measurements (5.00%)\n  5 (5.00%) high mild\nformat/10_attrs/1000_events\n                        time:   [720.96 µs 725.68 µs 730.81 µs]\n                        change: [−15.540% −13.383% −11.371%] (p = 0.00 < 0.05)\n                        Performance has improved.\nFound 9 outliers among 100 measurements (9.00%)\n  1 (1.00%) low mild\n  5 (5.00%) high mild\n  3 (3.00%) high severe\n\nencode_and_format/0_attrs/100_events\n                        time:   [32.698 µs 32.914 µs 33.147 µs]\n                        change: [−9.4066% −7.8944% −6.3427%] (p = 0.00 < 0.05)\n                        Performance has improved.\nFound 8 outliers among 100 measurements (8.00%)\n  2 (2.00%) low mild\n  3 (3.00%) high mild\n  3 (3.00%) high severe\nencode_and_format/3_attrs/100_events\n                        time:   [48.927 µs 49.498 µs 50.133 µs]\n                        change: [−7.2473% −5.1069% −2.7211%] (p = 0.00 < 0.05)\n                        Performance has improved.\nFound 10 outliers among 100 measurements (10.00%)\n  3 (3.00%) high mild\n  7 (7.00%) high severe\nencode_and_format/10_attrs/100_events\n                        time:   [95.328 µs 96.088 µs 96.970 µs]\n                        change: [−6.3169% −4.9414% −3.6501%] (p = 0.00 < 0.05)\n                        Performance has improved.\nFound 5 outliers among 100 measurements (5.00%)\n  4 (4.00%) high mild\n  1 (1.00%) high severe\nencode_and_format/0_attrs/1000_events\n                        time:   [326.65 µs 328.86 µs 331.27 µs]\n                        change: [−41.188% −39.915% −38.764%] (p = 0.00 < 0.05)\n                        Performance has improved.\nFound 7 outliers among 100 measurements (7.00%)\n  6 (6.00%) high mild\n  1 (1.00%) high severe\nencode_and_format/3_attrs/1000_events\n                        time:   [500.59 µs 504.82 µs 509.33 µs]\n                        change: [−50.787% −48.877% −47.483%] (p = 0.00 < 0.05)\n                        Performance has improved.\nFound 4 outliers among 100 measurements (4.00%)\n  3 (3.00%) high mild\n  1 (1.00%) high severe\nencode_and_format/10_attrs/1000_events\n                        time:   [944.34 µs 951.79 µs 960.38 µs]\n                        change: [−55.389% −54.741% −54.065%] (p = 0.00 < 0.05)\n                        Performance has improved.\nFound 4 outliers among 100 measurements (4.00%)\n  3 (3.00%) high mild\n  1 (1.00%) high severe\n```\n\n---------\n\nCo-authored-by: Laurent Quérel <laurent.querel@gmail.com>",
          "timestamp": "2026-01-09T23:01:40Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/002f4368ddd47cc05a69bd93e39b7f27850d9bc7"
        },
        "date": 1768213947448,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 41.43962443578216,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 42.20262064197531,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.788020833333334,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.27734375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 108330.96088528994,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108330.96088528994,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001314,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2210224.3535422664,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2095285.303842952,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.16923075914382935,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 44.81164782467225,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 47.6430941260257,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.53697916666667,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.31640625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 108329.56707538468,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108146.24011571864,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002086,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2416149.681124103,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 739807.3306205507,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
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
          "id": "f72798b2f168c0a7c2f469533ade55e6b1bd07c3",
          "message": "docs: Add architecture and configuration doc for mTLS/TLS for exporter and receiver.  (#1773)\n\nAdds comprehensive documentation for TLS/mTLS support in OTLP/OTAP\nreceivers and exporters.\n\n  ## Changes\n\n- **Configuration Guide**: User-facing documentation covering TLS/mTLS\nsetup, certificate hot-reload, configuration examples, security best\npractices, and troubleshooting\n- **Architecture Guide**: Developer-focused documentation covering\ndesign principles, component architecture, certificate reload\nmechanisms, performance characteristics, and future enhancements\n\nNote - Documentation was drafted using LLM , and then I validated\nagainst the code to ensure it is consistent.\n\n---------\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>\nCo-authored-by: Laurent Quérel <laurent.querel@gmail.com>",
          "timestamp": "2026-01-13T22:57:12Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/f72798b2f168c0a7c2f469533ade55e6b1bd07c3"
        },
        "date": 1768386877817,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 44.90829852067914,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 46.850642379664684,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 40.09348958333333,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.4140625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 108330.6232622414,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108330.6232622414,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001501,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2419619.0857956666,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 741006.2725056409,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 41.44100115569541,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 42.26820823847402,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.37890625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.0859375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 108332.09834741217,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108332.09834741217,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000684,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2212473.0042781876,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2096673.7642329189,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
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
          "id": "4b646461dc3070dbe85c5cbc3051ddd08d7331f3",
          "message": "start using thiserror instead of string to avoid using format (#1787)",
          "timestamp": "2026-01-15T00:27:58Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/4b646461dc3070dbe85c5cbc3051ddd08d7331f3"
        },
        "date": 1768440022519,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 44.49458858246362,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 46.9929019732674,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 40.301432291666664,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.296875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 108331.16490451583,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108331.16490451583,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001201,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2419641.9899784788,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 740619.8637206474,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 44.00872543535554,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 44.76924548802096,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 38.158203125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 39.37890625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106665.37423788215,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106665.37423788215,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000727,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2204351.809492737,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2092147.278528096,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
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
          "id": "05ea7a92fc0bc4cb1c3e85ab51fb390eb84a89ee",
          "message": "[otap-df-quiver] Quiver Subscriber API; quiver-e2e test tool (#1764)\n\n- Subscriber Registry & Lifecycle Management — Added a subscriber system\nto enable multiple independent consumers to track progress through the\nsegment stream. Includes registration, activation/deactivation, and\nbundle claiming with RAII handles (BundleHandle) for ack/reject/defer\nsemantics.\n- Durable Progress File Format — Implemented a versioned binary format\n(`quiver.sub.<id>`) with CRC32 validation for crash-safe progress\npersistence. Uses atomic write-fsync-rename for durability. Supports\nper-bundle ack tracking via bitmaps for out-of-order delivery scenarios.\n- Disk Budget & Backpressure — Added `DiskBudget` for enforcing storage\ncaps with two retention policies: `Backpressure` (slow down ingestion)\nand `DropOldest` (force-complete old segments). Supports reserved\nheadroom for WAL rotation and segment finalization.\n- Engine API Unification — Extended `QuiverEngine` to be the entry point\nfor ingestion, subscription, and maintenance. Added `maintain()` method\nfor periodic progress flush + segment cleanup, builder pattern for\nconfiguration, and blocking `next_bundle_blocking()` with condvar-based\nwakeup.\n- Add a `quiver-e2e` crate for stress testing the persistence flow.\nFeatures concurrent ingest/consume, support for multiple Quiver engine\ninstances, TUI dashboard with real-time metrics, configurable disk\nbudgets, jemalloc memory tracking, and subscriber delay simulation.",
          "timestamp": "2026-01-15T15:57:46Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/05ea7a92fc0bc4cb1c3e85ab51fb390eb84a89ee"
        },
        "date": 1768496161581,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 44.644676771433936,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 45.560826917866876,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 38.705859375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 39.5390625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 108332.02432137278,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108332.02432137278,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000725,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2401841.890730289,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 740107.9852426025,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 41.842477198575864,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 42.6714485185472,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 40.231510416666666,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.17578125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 108320.5858057271,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108320.5858057271,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.007061,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2210665.4753218205,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2096148.2162080344,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
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
          "id": "94af57b4abe8ecb93838572f259645cc6ea9b5a7",
          "message": "Scale and Saturation test update (#1788)\n\nLocal run output is shown below. The same is uploaded to usual charts,\nso we can see how linearly we scale with CPU cores.\n\nThe saturation-tests will be refactored in future, to focus just on the\nscaling aspects (and probably renamed as scaling-tests).\n\n\n```txt\n==============================================\nAnalyzing Scaling Efficiency\n==============================================\n\nFound: 1 core(s) -> 181,463 logs/sec\nFound: 2 core(s) -> 257,679 logs/sec\nFound: 4 core(s) -> 454,159 logs/sec\n\n================================================================================\nSATURATION/SCALING TEST RESULTS - SCALING ANALYSIS\n================================================================================\n\nGoal: Verify shared-nothing architecture with linear CPU scaling\nBaseline (1 core): 181,463 logs/sec\n\n--------------------------------------------------------------------------------\nCores    Throughput (logs/sec)     Expected (linear)    Scaling Efficiency\n--------------------------------------------------------------------------------\n1        181,463                   181,463              100.00% ✅\n2        257,679                   362,927              71.00% 🟠\n4        454,159                   725,853              62.57% 🔴\n--------------------------------------------------------------------------------\n\nSUMMARY:\n  • Average Scaling Efficiency: 77.86%\n  • Minimum Scaling Efficiency: 62.57%\n  • Maximum Throughput (4 cores): 454,159 logs/sec\n  • Speedup (4 cores vs 1 core): 2.5x\n\n🟠 ACCEPTABLE: The engine shows reasonable scaling.\n   Some contention or overhead present.\n\n================================================================================\n```\n\n---------\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2026-01-15T23:41:59Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/94af57b4abe8ecb93838572f259645cc6ea9b5a7"
        },
        "date": 1768526575233,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.0615384615957737,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 44.26227387453459,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 45.44197946550653,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 38.733463541666666,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 39.40625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 108331.21004161652,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108264.54468159091,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001176,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2383670.0371666895,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 728447.7811564978,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 41.19612210942357,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 41.93365837854549,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 40.43177083333333,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.61328125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 108331.51516940373,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108331.51516940373,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001007,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2204105.424396566,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2093610.608617066,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
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
          "id": "8e6891bb6def12af916041036a75eed2327c639a",
          "message": "Add service::telemetry::logs::providers settings for internal logging setup (#1795)\n\nPart of https://github.com/open-telemetry/otel-arrow/pull/1771.\n\nPart of https://github.com/open-telemetry/otel-arrow/issues/1736.\n\nAs documented in https://github.com/open-telemetry/otel-arrow/pull/1741.\n\n~Updates that document to match this change reflecting the prototype in\n#1771.~\n\nRevised relative to #1771.\n\nAdds LoggingProviders (choice of default logging provider for global,\nengine, and internal-telemetry threads).\nAdds ProviderMode with names to select instrumentation behavior, with\n`its` referring to internal telemetry system.\n\nNote: These settings are somehow not ideally placed. They belong also in\nthe top-level settings, or with observed_state settings. However, since\nlogging is configured with resource and level, which are part of the\nservice::telemetry config area presently, we use that structure. After\nthe bulk of #1736 is finished we can restructure.",
          "timestamp": "2026-01-16T05:28:35Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/8e6891bb6def12af916041036a75eed2327c639a"
        },
        "date": 1768582859400,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.04615384712815285,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 45.71268624350206,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 46.80046161490683,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 38.72890625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 40.73828125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 108319.14894077953,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108269.15548742225,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.007857,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2418226.9299756787,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 740964.0858860704,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 41.36433755809286,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 42.312122343496725,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 40.511067708333336,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.46484375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 108331.54947381867,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108331.54947381867,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000988,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2207121.1886588507,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2093279.399139125,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
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
          "id": "c68e70eda406b6341cbd0ae73cf4521a56639d47",
          "message": "Update batch size variation perf tests (#1809)\n\nModified to use 10, 100, 512, 1024, 4096, 8192 as sizes.\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2026-01-16T23:41:49Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/c68e70eda406b6341cbd0ae73cf4521a56639d47"
        },
        "date": 1768612818145,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 41.52410960204646,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 42.31098854097474,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.688411458333334,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.22265625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 108331.59280574224,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108331.59280574224,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000964,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2205121.422386031,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2092855.5182367044,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 44.20250562567754,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 46.17122781317662,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.82434895833333,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.43359375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 108328.99114627155,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108328.99114627155,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002405,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2420765.964962733,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 740237.7581636801,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
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
          "id": "8470d2d442782c9e6dadf2b9379160f88ccc2c39",
          "message": "Split opentelemetry_client into otel_sdk, tracing_init, and ITS parts (#1808)\n\nPart of https://github.com/open-telemetry/otel-arrow/pull/1771.\n\nPart of https://github.com/open-telemetry/otel-arrow/issues/1736.\n\nThis is a non-functional refactoring of `opentelemetry_client.rs` into\nother places. This will make it clearer what changes in #1771 and what\nis just moving around.\n\nMoves runtime elements into the InternalTelemetrySystem, simplifies\nsetup for the controller where logs/metrics were separated.\n\nMoves OTel-SDK specific pieces into `otel_sdk` module, separates the\nTokio `tracing` setup.\n\n---------\n\nCo-authored-by: Utkarsh Umesan Pillai <66651184+utpilla@users.noreply.github.com>",
          "timestamp": "2026-01-17T02:49:23Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/8470d2d442782c9e6dadf2b9379160f88ccc2c39"
        },
        "date": 1768668675404,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.1538461595773697,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 45.65362930824806,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 47.76442987951807,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 38.764713541666666,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 39.328125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 108329.26015315157,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108162.59975291595,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002256,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2411772.183843432,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 739573.7246707208,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 42.19700100888949,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 43.02104754670181,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 40.32734375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.22265625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 108330.58715294901,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108330.58715294901,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001521,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2203673.2981893388,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2088921.4735887547,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
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
          "id": "8470d2d442782c9e6dadf2b9379160f88ccc2c39",
          "message": "Split opentelemetry_client into otel_sdk, tracing_init, and ITS parts (#1808)\n\nPart of https://github.com/open-telemetry/otel-arrow/pull/1771.\n\nPart of https://github.com/open-telemetry/otel-arrow/issues/1736.\n\nThis is a non-functional refactoring of `opentelemetry_client.rs` into\nother places. This will make it clearer what changes in #1771 and what\nis just moving around.\n\nMoves runtime elements into the InternalTelemetrySystem, simplifies\nsetup for the controller where logs/metrics were separated.\n\nMoves OTel-SDK specific pieces into `otel_sdk` module, separates the\nTokio `tracing` setup.\n\n---------\n\nCo-authored-by: Utkarsh Umesan Pillai <66651184+utpilla@users.noreply.github.com>",
          "timestamp": "2026-01-17T02:49:23Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/8470d2d442782c9e6dadf2b9379160f88ccc2c39"
        },
        "date": 1768699461143,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 42.059956862910525,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 42.86969386468953,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 40.28098958333333,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.4609375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 108332.14167977485,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108332.14167977485,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00066,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2208281.320381752,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2092388.6360445942,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.1846153885126114,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 46.24609237406859,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 47.501377328788344,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 38.864453125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 40.9296875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 108330.90130459904,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108130.90579449825,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001347,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2414602.8322027475,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 739523.83108612,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
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
          "id": "d8a0d6d381f1a9f2968c182c88920cb4ded93cc0",
          "message": "Create entity and expose entity keys via thread locals and task locals (#1785)\n\nThe engine now creates the following entities:\n\n* **Pipeline** -> Stored in a thread local associated with the pipeline\nthread.\n* **Node** -> Stored in the task local of the node.\n* **Channel**\n  * **Sender entity** stored in the task local of the sender node.\n  * **Receiver entity** stored in the task local of the receiver node.\n\nAn entity cleanup mechanism is in place. A unit test has been added to\nvalidate this cleanup process.\n\nThe final goal is to be able to use these entities directly when\nreporting metric sets and events. This allows us to report the\nattributes of all our entities using a simple numerical ID.\n\nCloses https://github.com/open-telemetry/otel-arrow/issues/1791",
          "timestamp": "2026-01-18T07:23:23Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/d8a0d6d381f1a9f2968c182c88920cb4ded93cc0"
        },
        "date": 1768755076022,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 41.10267829839701,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 41.73680455163884,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 40.04661458333333,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.109375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106665.22846383622,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106665.22846383622,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000809,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2209959.383442467,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2090483.3439495286,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.140625,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 43.95686759277965,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 45.04354546918551,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.2125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.61328125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106663.41165488766,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106813.40707752734,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001831,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2387314.488706712,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 735998.8031393561,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
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
          "id": "d8a0d6d381f1a9f2968c182c88920cb4ded93cc0",
          "message": "Create entity and expose entity keys via thread locals and task locals (#1785)\n\nThe engine now creates the following entities:\n\n* **Pipeline** -> Stored in a thread local associated with the pipeline\nthread.\n* **Node** -> Stored in the task local of the node.\n* **Channel**\n  * **Sender entity** stored in the task local of the sender node.\n  * **Receiver entity** stored in the task local of the receiver node.\n\nAn entity cleanup mechanism is in place. A unit test has been added to\nvalidate this cleanup process.\n\nThe final goal is to be able to use these entities directly when\nreporting metric sets and events. This allows us to report the\nattributes of all our entities using a simple numerical ID.\n\nCloses https://github.com/open-telemetry/otel-arrow/issues/1791",
          "timestamp": "2026-01-18T07:23:23Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/d8a0d6d381f1a9f2968c182c88920cb4ded93cc0"
        },
        "date": 1768785801861,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 41.195444474377844,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 42.27045058859784,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.71692708333333,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.5,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106665.12357787891,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106665.12357787891,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000868,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2204582.679551045,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2084178.1262788214,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.03125,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 44.602389928668565,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 46.217035690402476,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.408984375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.33984375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106665.30668400644,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106631.97377566769,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000765,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2420565.3922765157,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 740227.1568466873,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
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
          "id": "2c3976c9672536835e94dae07a4cc7f26333276e",
          "message": "user lowercase for event names (#1816)\n\nhttps://github.com/open-telemetry/otel-arrow/blob/main/rust/otap-dataflow/docs/telemetry/events-guide.md#event-naming\n\nMoving to lowercase. We are not fully following the guided name yet.\nWill tackle that one module at a time in follow ups.\n\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-01-19T12:14:46Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/2c3976c9672536835e94dae07a4cc7f26333276e"
        },
        "date": 1768841557146,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 41.47521917157163,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 42.36253627254509,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 40.14856770833333,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.1171875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106665.46668016652,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106665.46668016652,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000675,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2208126.5187314856,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2090072.0871590422,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.171875,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 45.886419207139895,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 47.11203243645381,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 38.72513020833333,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 40.79296875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106665.36890467833,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106482.03780187342,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00073,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2419358.0894540013,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 740906.1941681218,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
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
          "id": "86b03dcd2ab9007d29e7cb0de6d1fcf86c9ead6b",
          "message": "PerfTest - include OTAP to OTAP in saturation/scaling test (#1815)\n\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-01-19T21:43:46Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/86b03dcd2ab9007d29e7cb0de6d1fcf86c9ead6b"
        },
        "date": 1768872052169,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.03125,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 44.361240628786184,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 45.75881243898695,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.773828125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.81640625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106662.97079472862,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106629.63861635527,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002079,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2422189.7644875487,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 740285.099500431,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 41.529331873486505,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 42.46927878106964,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 40.645182291666664,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.8359375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106665.29957307714,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106665.29957307714,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000769,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2214969.909873587,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2099634.8779401532,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Sachin Bansal",
            "username": "Apostlex0",
            "email": "sachinnb999@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "107ef6bc4736d44aa80c9926082affb44d5f66c0",
          "message": "feat: filter tests updated to use OPL parser (#1836)\n\ncloses #1790\n\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-01-20T15:57:44Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/107ef6bc4736d44aa80c9926082affb44d5f66c0"
        },
        "date": 1768928082416,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.328125,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 41.392503499292914,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 41.91250182604666,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 40.57825520833333,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106663.83296417093,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108080.46199572632,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001594,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2219755.1232848046,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2101158.6406194004,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.0317460298538208,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 43.83479737698093,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 45.000680950685144,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.19296875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 40.8984375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104994.52803518057,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 105027.8596313822,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003127,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2385084.3710239967,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 728030.9883432572,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
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
          "id": "e4c170b3704bac31d91ff764fd8ad9eb2dad51e3",
          "message": "Replace uses of log:: with otel_ macros in crates/engine, crates/otap (#1843)\n\nPart of https://github.com/open-telemetry/otel-arrow/pull/1771.\n\nPart of https://github.com/open-telemetry/otel-arrow/issues/1736.\n\nOverlaps with #1841 by copying the file\ncrates/telemetry/src/internal_events.rs to extend the otel_xxx macros to\nfull Tokio syntax, to replace uses of log formatting as needed.\n\nAfter this, #1841 can remove \"log\" from the workspace Cargo.toml b/c\ncrates/state will have the remaining \"log\" references fixed there.",
          "timestamp": "2026-01-20T23:18:00Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/e4c170b3704bac31d91ff764fd8ad9eb2dad51e3"
        },
        "date": 1768958569637,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 45.30857807211828,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 46.92450723195085,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 38.84609375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 40.8125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106665.0151366823,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106665.0151366823,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000929,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2424501.1219938393,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 741213.5763354733,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 40.80289863964771,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 41.45650217209949,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.571223958333334,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.2421875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106663.3369928302,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106663.3369928302,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001873,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2221653.022768305,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2104548.9076630124,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
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
          "id": "0ca864735f0e6a57139adb2670e4cb21ce2080f9",
          "message": "Add starter PR template (#1846)\n\n# Change Summary\n\nProposing adding the following PR template as suggested\n[here](https://github.com/open-telemetry/otel-arrow/issues/1749#issuecomment-3747119300).\n\nOpen to reducing verbosity.\n\n## What issue does this PR close?\n\n* Closes #1845 \n\n## How are these changes tested?\n\nN/A\n\n## Are there any user-facing changes?\n\nNo\n\n---------\n\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-01-21T13:45:47Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/0ca864735f0e6a57139adb2670e4cb21ce2080f9"
        },
        "date": 1769014727021,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.78125,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 40.90148295263801,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 41.761460384763964,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 40.01901041666667,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106665.20179789531,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 107498.52368694136,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000824,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2208370.3029412264,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2093727.770819098,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 43.58961577811116,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 46.09037272685221,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.98645833333333,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.4375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106664.92625062,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106664.92625062,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000979,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2388606.7898133015,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 727912.1116997167,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
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
          "id": "9ef8217d57a10ef141472c3766b2d778fc296928",
          "message": "Internal logging provider setup; console_async support (#1841)\n\nPart of https://github.com/open-telemetry/otel-arrow/pull/1771.\n\nPart of https://github.com/open-telemetry/otel-arrow/issues/1736.\n\nImplements 4 of the 5 ProviderMode values.\n\nUses the ObservedStateStore's associated thread and channel to process\nconsole_async messages.\n\nReplaces most of #1771.\n\nUndoes portions of #1818:\n\n- ObservedEvent is an enum for Engine, Log events\n- Engine events return to `Option<String>` message, no structured\nmessage\n- Removes info_event! and error_event! structured message constructor\nmacros\n- Moves LogRecord::Serialize support to where it's used\n\nAdds new LoggingProviders selector `admin` to configure how the admin\nthreads use internal logging. The new setting defaults to ConsoleDirect,\ni.e., the admin components will use synchronous console logging.\n\nConfigures the Tokio tracing subscriber globally, in engine threads, and\nin admin threads according to the ProviderMode.\n\nThe asynchronous tracing subscriber (which sends to console_async; will\nsend to ITS in the future) uses the `internal` provider mode itself as a\nfallback. However, it does this directly, choosing the Noop or\nConsoleDirect modes, OpenTelemetry mode is not supported here.\n\n~Resolves a TODO about inconsistency in the otel_xxx! macros. These now\nsupport full Tokio syntax following raw_error!~\nEDIT: portions of this PR were moved into #1843. This PR removes the\ntop-level `log` dependency.\n\n---------\n\nCo-authored-by: Cijo Thomas <cithomas@microsoft.com>\nCo-authored-by: Lalit Kumar Bhasin <lalit_fin@yahoo.com>",
          "timestamp": "2026-01-21T20:51:55Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/9ef8217d57a10ef141472c3766b2d778fc296928"
        },
        "date": 1769045016790,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 41.904744999694415,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 42.84109818294382,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.64505208333333,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.3671875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106664.19027971567,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106664.19027971567,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001393,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2213519.340421838,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2095731.0085860854,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 45.41180068506819,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 46.655135436004,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.008723958333334,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.5078125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106662.71481308284,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106662.71481308284,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002223,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2417405.3896338255,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 739982.5604605401,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
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
          "id": "2fae2f7dafe88504069a439a3d9ef89ea49f09ff",
          "message": "Console exporter for OTAP/OTLP logs (#1849)\n\nPart of https://github.com/open-telemetry/otel-arrow/pull/1771.\n\nPart of https://github.com/open-telemetry/otel-arrow/issues/1736.\n\nUses the new internal logging support to format OTLP logs data. This\nprints RESOURCE and SCOPE lines with ASCII or Unicode pipe structures to\nidentify the OTLP hierarchy:\n\n```\n2026-01-21T03:12:22.165Z  RESOURCE   v1.Resource: [fake_data_generator=v1]\n2026-01-21T03:12:22.165Z  │ SCOPE    v1.InstrumentationScope:\n2026-01-21T03:12:22.165Z  │ ├─ INFO  session.start:  [session.id=00112233-4455-6677-8899-aabbccddeeff, session.previous_id=00112233-4455-6677-8899-aabbccddeeff]\n2026-01-21T03:12:22.165Z  │ ├─ INFO  session.end:  [session.id=00112233-4455-6677-8899-aabbccddeeff]\n2026-01-21T03:12:22.165Z  │ ├─ INFO  device.app.lifecycle:  [ios.app.state=active, android.app.state=created]\n2026-01-21T03:12:22.165Z  │ ├─ INFO  rpc.message:  [rpc.message.type=SENT, rpc.message.id=42, rpc.message.compressed_size=42, rpc.message.uncompressed_size=42]\n```\n\n---------\n\nCo-authored-by: Utkarsh Umesan Pillai <66651184+utpilla@users.noreply.github.com>",
          "timestamp": "2026-01-22T04:56:14Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/2fae2f7dafe88504069a439a3d9ef89ea49f09ff"
        },
        "date": 1769100868512,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 42.07800340507753,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 43.14643694583558,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 40.09817708333333,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.640625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106665.20179789531,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106665.20179789531,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000824,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2204477.274573766,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2084022.9140298883,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.140625,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 45.90754093180839,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 47.24900739550336,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.1296875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.4296875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106655.78244406825,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106505.79775000627,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.006123,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2417446.6961925053,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 742361.406955197,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
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
          "id": "c9f4e5d1a3249bebfe87f9d1d74c7d91f2ef171b",
          "message": "Add few logs to various components to expose shutdown issue (#1869)\n\n# Change Summary\n\nAdds/improves few internal logs to make the engine more observable. \n\n## How are these changes tested?\n\nLocal, manual runs\n\n## Are there any user-facing changes?\n\nBetter logs!",
          "timestamp": "2026-01-23T00:01:10Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/c9f4e5d1a3249bebfe87f9d1d74c7d91f2ef171b"
        },
        "date": 1769131745297,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.5384615659713745,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 24.004257893101748,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 24.701740009254973,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 36.9421875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 38.10546875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 108330.58715294901,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 109997.21157068669,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001521,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 723653.2505279694,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 722576.0460801194,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.5384615659713745,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 28.7276444938709,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 31.49835074526906,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 36.570963541666664,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 39.25390625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 108329.99315854428,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 109996.6084379065,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00185,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 763862.0856045383,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 303200.9985454679,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
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
          "id": "c9f4e5d1a3249bebfe87f9d1d74c7d91f2ef171b",
          "message": "Add few logs to various components to expose shutdown issue (#1869)\n\n# Change Summary\n\nAdds/improves few internal logs to make the engine more observable. \n\n## How are these changes tested?\n\nLocal, manual runs\n\n## Are there any user-facing changes?\n\nBetter logs!",
          "timestamp": "2026-01-23T00:01:10Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/c9f4e5d1a3249bebfe87f9d1d74c7d91f2ef171b"
        },
        "date": 1769187723878,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.5384615659713745,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 23.821562008389837,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 24.521765394370554,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 37.24856770833333,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 38.07421875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 108331.96835053212,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 109998.61401746338,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000756,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 720551.998543657,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 719046.0578252164,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.5384615659713745,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 28.566733859244586,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 30.89637487684729,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 36.78203125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 39.49609375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 108331.44114416135,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 109998.07870022538,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001048,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 748315.3055809167,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 300337.6773216377,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
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
          "id": "e8c9e29cbe5ff77d4f289839c42cf884d39bccdb",
          "message": "Add support for fan-in DAG topologies (#1859)\n\n# Change Summary\n\nThe engine currently supports simple and \"balanced\" fan-out topologies\nbut does not yet handle broadcast fan-out, fan-in or combined fan-in and\nfan-out connections, even though the configuration model allows them.\n\nThe lack of fan-in support prevents multiple upstream nodes from feeding\nthe same downstream node. Supporting these topologies is required to\nenable more complex and expressive pipeline graphs.\n\nThis PR doesn't try to solve the broadcast fan-out limit.\n\n**Important note**: I refactored method `PipelineFactory::build` because\nits complexity had become difficult to follow. Most of the changes are\ndue to this refactoring rather than to the fan in support itself.\n\n## What issue does this PR close?\n\n* Closes #1860\n\n## How are these changes tested?\n\nA unit test has been added into `pipeline_tests.rs` and I also did a\nmanual test.\n\n## Are there any user-facing changes?\n\nNo change in the configuration file.",
          "timestamp": "2026-01-23T23:30:14Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/e8c9e29cbe5ff77d4f289839c42cf884d39bccdb"
        },
        "date": 1769217699878,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.453125,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 41.91650890327819,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 43.11528554553932,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 37.67578125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 38.48828125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106665.37601561687,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 107148.70350068764,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000726,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2381077.281837781,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 528939.8610853278,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 40.936247462352235,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 41.76066734365325,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.305859375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.0234375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106665.26579617588,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106665.26579617588,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000788,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2142381.5857148194,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2029376.7825082047,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
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
          "id": "6092d5db8907489526ecea718206b851cc69c7da",
          "message": "Internal Telemetry Receiver logging provider (#1861)\n\n## What issue does this PR close?\n\nFinal part of #1771.\n\nFixes #1736.\n\n## How are these changes tested?\n\nNew tests. A new configs/internal-telemetry.yaml demonstrates the ITR\nconfiguration with the new console exporter.\n\n## Are there any user-facing changes?\n\nYes. See README.md update.\n\n---------\n\nCo-authored-by: Cijo Thomas <cithomas@microsoft.com>\nCo-authored-by: Lalit Kumar Bhasin <lalit_fin@yahoo.com>\nCo-authored-by: Utkarsh Umesan Pillai <66651184+utpilla@users.noreply.github.com>\nCo-authored-by: Laurent Quérel <laurent.querel@gmail.com>",
          "timestamp": "2026-01-24T02:49:22Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/6092d5db8907489526ecea718206b851cc69c7da"
        },
        "date": 1769273480166,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.25,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 41.50940181040229,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 41.9849491311412,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 37.588802083333334,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 39.13671875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106665.21957518777,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 107998.53481987762,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000814,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2145060.321122062,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2056522.3378077394,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.65625,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 42.65802313661331,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 43.55692839093791,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 40.037760416666664,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 42.47265625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106665.21957518777,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 107365.21007864994,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000814,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2436138.2314095967,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 532716.9729672844,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
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
          "id": "58180138e1dfd8118f9b7d39cad784272aa50b74",
          "message": "perftest- temporarily remove saturation test with 24 core (#1884)\n\nI am observing similar issue to\nhttps://github.com/open-telemetry/otel-arrow/issues/1870 in the OTLP to\nOTLP scenario in loadtest - for the 24 core SUT, we use 72 core\nLoad-generator, and the load-generator is not shutting down properly. It\nis entirely possible that 72 pipelines instances would need more time to\nshutdown; until this can be investigated, its best to temporarily remove\nthis scenario.\n\nTo unblock perf tests, disabling the 24 core test temporarily. I'll\ninvestigate a proper fix next week.",
          "timestamp": "2026-01-24T18:42:13Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/58180138e1dfd8118f9b7d39cad784272aa50b74"
        },
        "date": 1769304349531,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.5625,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 41.4536709620559,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 42.375852027560576,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.216015625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.1328125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106664.12450503263,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108330.75145042376,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00143,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2139641.9814426624,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1989393.5785861502,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 42.2088387519741,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 43.91828260340445,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 38.215234375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 40.52734375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106658.77391739677,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106658.77391739677,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00444,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2440643.8656037576,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 527772.7061695316,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
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
          "id": "58180138e1dfd8118f9b7d39cad784272aa50b74",
          "message": "perftest- temporarily remove saturation test with 24 core (#1884)\n\nI am observing similar issue to\nhttps://github.com/open-telemetry/otel-arrow/issues/1870 in the OTLP to\nOTLP scenario in loadtest - for the 24 core SUT, we use 72 core\nLoad-generator, and the load-generator is not shutting down properly. It\nis entirely possible that 72 pipelines instances would need more time to\nshutdown; until this can be investigated, its best to temporarily remove\nthis scenario.\n\nTo unblock perf tests, disabling the 24 core test temporarily. I'll\ninvestigate a proper fix next week.",
          "timestamp": "2026-01-24T18:42:13Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/58180138e1dfd8118f9b7d39cad784272aa50b74"
        },
        "date": 1769359799786,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 42.277724107871585,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 43.32466178217822,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 38.14609375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 40.3984375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106664.76981151018,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106664.76981151018,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001067,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2415072.3832275346,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 524860.2639772521,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 41.164988329829036,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 42.161851911730544,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 37.57591145833333,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 39.48828125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106665.11646697401,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106665.11646697401,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000872,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2141548.7814089884,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2007521.3042228993,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
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
          "id": "58180138e1dfd8118f9b7d39cad784272aa50b74",
          "message": "perftest- temporarily remove saturation test with 24 core (#1884)\n\nI am observing similar issue to\nhttps://github.com/open-telemetry/otel-arrow/issues/1870 in the OTLP to\nOTLP scenario in loadtest - for the 24 core SUT, we use 72 core\nLoad-generator, and the load-generator is not shutting down properly. It\nis entirely possible that 72 pipelines instances would need more time to\nshutdown; until this can be investigated, its best to temporarily remove\nthis scenario.\n\nTo unblock perf tests, disabling the 24 core test temporarily. I'll\ninvestigate a proper fix next week.",
          "timestamp": "2026-01-24T18:42:13Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/58180138e1dfd8118f9b7d39cad784272aa50b74"
        },
        "date": 1769390620912,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 42.03214000316875,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 44.35387349240781,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 38.565234375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 40.953125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106664.56715243655,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106664.56715243655,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001181,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2419381.424273522,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 529395.4622667482,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 41.32090242154308,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 42.020201603225054,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.713802083333334,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.4140625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106664.97069363264,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106664.97069363264,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000954,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2175010.468067793,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2029788.7358099844,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
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
          "id": "3f0c85c4d65a91562de3165088edececc378f0eb",
          "message": "fix(deps): update module go.opentelemetry.io/collector/pdata to v1.50.0 (#1890)\n\nThis PR contains the following updates:\n\n| Package | Change |\n[Age](https://docs.renovatebot.com/merge-confidence/) |\n[Confidence](https://docs.renovatebot.com/merge-confidence/) |\n|---|---|---|---|\n|\n[go.opentelemetry.io/collector/pdata](https://redirect.github.com/open-telemetry/opentelemetry-collector)\n| `v1.49.0` → `v1.50.0` |\n![age](https://developer.mend.io/api/mc/badges/age/go/go.opentelemetry.io%2fcollector%2fpdata/v1.50.0?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/go/go.opentelemetry.io%2fcollector%2fpdata/v1.49.0/v1.50.0?slim=true)\n|\n\n---\n\n### Release Notes\n\n<details>\n<summary>open-telemetry/opentelemetry-collector\n(go.opentelemetry.io/collector/pdata)</summary>\n\n###\n[`v1.50.0`](https://redirect.github.com/open-telemetry/opentelemetry-collector/blob/HEAD/CHANGELOG.md#v1500v01440)\n\n##### 🛑 Breaking changes 🛑\n\n- `pkg/exporterhelper`: Change verbosity level for\notelcol\\_exporter\\_queue\\_batch\\_send\\_size metric to detailed.\n([#&#8203;14278](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/14278))\n- `pkg/service`: Remove deprecated\n`telemetry.disableHighCardinalityMetrics` feature gate.\n([#&#8203;14373](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/14373))\n- `pkg/service`: Remove deprecated `service.noopTracerProvider` feature\ngate.\n([#&#8203;14374](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/14374))\n\n##### 🚩 Deprecations 🚩\n\n- `exporter/otlp_grpc`: Rename `otlp` exporter to `otlp_grpc` exporter\nand add deprecated alias `otlp`.\n([#&#8203;14403](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/14403))\n- `exporter/otlp_http`: Rename `otlphttp` exporter to `otlp_http`\nexporter and add deprecated alias `otlphttp`.\n([#&#8203;14396](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/14396))\n\n##### 💡 Enhancements 💡\n\n- `cmd/builder`: Avoid duplicate CLI error logging in generated\ncollector binaries by relying on cobra's error handling.\n([#&#8203;14317](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/14317))\n\n- `cmd/mdatagen`: Add the ability to disable attributes at the metric\nlevel and re-aggregate data points based off of these new dimensions\n([#&#8203;10726](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/10726))\n\n- `cmd/mdatagen`: Add optional `display_name` and `description` fields\nto metadata.yaml for human-readable component names\n([#&#8203;14114](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/14114))\nThe `display_name` field allows components to specify a human-readable\nname in metadata.yaml.\nWhen provided, this name is used as the title in generated README files.\nThe `description` field allows components to include a brief description\nin generated README files.\n\n- `cmd/mdatagen`: Validate stability level for entities\n([#&#8203;14425](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/14425))\n\n- `pkg/xexporterhelper`: Reenable batching for profiles\n([#&#8203;14313](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/14313))\n\n- `receiver/nop`: add profiles signal support\n([#&#8203;14253](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/14253))\n\n##### 🧰 Bug fixes 🧰\n\n- `pkg/exporterhelper`: Fix reference count bug in partition batcher\n([#&#8203;14444](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/14444))\n\n<!-- previous-version -->\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - \"before 8am on Monday\" (UTC),\nAutomerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0Mi45Mi4xIiwidXBkYXRlZEluVmVyIjoiNDIuOTIuMSIsInRhcmdldEJyYW5jaCI6Im1haW4iLCJsYWJlbHMiOlsiZGVwZW5kZW5jaWVzIl19-->\n\n---------\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: otelbot <197425009+otelbot@users.noreply.github.com>\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-01-26T16:09:46Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/3f0c85c4d65a91562de3165088edececc378f0eb"
        },
        "date": 1769446417233,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.5625,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 41.24837803103433,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 41.932966214549936,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.046484375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.703125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106665.17690969583,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108331.82029890982,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000838,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2164760.7238471727,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2031018.836179772,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.734375,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 42.37410291209922,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 43.698734833371994,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 37.831380208333336,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 38.76953125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106664.79114408905,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 107448.11070405346,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001055,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2428937.0139745567,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 533951.6248207887,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
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
          "id": "3f0c85c4d65a91562de3165088edececc378f0eb",
          "message": "fix(deps): update module go.opentelemetry.io/collector/pdata to v1.50.0 (#1890)\n\nThis PR contains the following updates:\n\n| Package | Change |\n[Age](https://docs.renovatebot.com/merge-confidence/) |\n[Confidence](https://docs.renovatebot.com/merge-confidence/) |\n|---|---|---|---|\n|\n[go.opentelemetry.io/collector/pdata](https://redirect.github.com/open-telemetry/opentelemetry-collector)\n| `v1.49.0` → `v1.50.0` |\n![age](https://developer.mend.io/api/mc/badges/age/go/go.opentelemetry.io%2fcollector%2fpdata/v1.50.0?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/go/go.opentelemetry.io%2fcollector%2fpdata/v1.49.0/v1.50.0?slim=true)\n|\n\n---\n\n### Release Notes\n\n<details>\n<summary>open-telemetry/opentelemetry-collector\n(go.opentelemetry.io/collector/pdata)</summary>\n\n###\n[`v1.50.0`](https://redirect.github.com/open-telemetry/opentelemetry-collector/blob/HEAD/CHANGELOG.md#v1500v01440)\n\n##### 🛑 Breaking changes 🛑\n\n- `pkg/exporterhelper`: Change verbosity level for\notelcol\\_exporter\\_queue\\_batch\\_send\\_size metric to detailed.\n([#&#8203;14278](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/14278))\n- `pkg/service`: Remove deprecated\n`telemetry.disableHighCardinalityMetrics` feature gate.\n([#&#8203;14373](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/14373))\n- `pkg/service`: Remove deprecated `service.noopTracerProvider` feature\ngate.\n([#&#8203;14374](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/14374))\n\n##### 🚩 Deprecations 🚩\n\n- `exporter/otlp_grpc`: Rename `otlp` exporter to `otlp_grpc` exporter\nand add deprecated alias `otlp`.\n([#&#8203;14403](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/14403))\n- `exporter/otlp_http`: Rename `otlphttp` exporter to `otlp_http`\nexporter and add deprecated alias `otlphttp`.\n([#&#8203;14396](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/14396))\n\n##### 💡 Enhancements 💡\n\n- `cmd/builder`: Avoid duplicate CLI error logging in generated\ncollector binaries by relying on cobra's error handling.\n([#&#8203;14317](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/14317))\n\n- `cmd/mdatagen`: Add the ability to disable attributes at the metric\nlevel and re-aggregate data points based off of these new dimensions\n([#&#8203;10726](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/10726))\n\n- `cmd/mdatagen`: Add optional `display_name` and `description` fields\nto metadata.yaml for human-readable component names\n([#&#8203;14114](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/14114))\nThe `display_name` field allows components to specify a human-readable\nname in metadata.yaml.\nWhen provided, this name is used as the title in generated README files.\nThe `description` field allows components to include a brief description\nin generated README files.\n\n- `cmd/mdatagen`: Validate stability level for entities\n([#&#8203;14425](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/14425))\n\n- `pkg/xexporterhelper`: Reenable batching for profiles\n([#&#8203;14313](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/14313))\n\n- `receiver/nop`: add profiles signal support\n([#&#8203;14253](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/14253))\n\n##### 🧰 Bug fixes 🧰\n\n- `pkg/exporterhelper`: Fix reference count bug in partition batcher\n([#&#8203;14444](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/14444))\n\n<!-- previous-version -->\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - \"before 8am on Monday\" (UTC),\nAutomerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0Mi45Mi4xIiwidXBkYXRlZEluVmVyIjoiNDIuOTIuMSIsInRhcmdldEJyYW5jaCI6Im1haW4iLCJsYWJlbHMiOlsiZGVwZW5kZW5jaWVzIl19-->\n\n---------\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: otelbot <197425009+otelbot@users.noreply.github.com>\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-01-26T16:09:46Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/3f0c85c4d65a91562de3165088edececc378f0eb"
        },
        "date": 1769477014822,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.7460317611694336,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 42.50066340658256,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 43.78882617100372,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.91875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.36328125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104995.44844730981,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 105778.74782461514,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002601,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2410959.2988734413,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 535827.0580996475,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.328125,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 41.35372543961949,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 42.139731648079305,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.656901041666664,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.3359375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106664.43204681529,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108081.06903493704,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001257,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2199023.3718009293,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2075271.3964558973,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
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
          "id": "e1c7a802b626d7c8a6061e9f1a3ced60ac9417eb",
          "message": "fix(deps): update all patch versions (#1894)\n\nThis PR contains the following updates:\n\n| Package | Change |\n[Age](https://docs.renovatebot.com/merge-confidence/) |\n[Confidence](https://docs.renovatebot.com/merge-confidence/) |\n|---|---|---|---|\n| [duckdb](https://redirect.github.com/duckdb/duckdb-python)\n([changelog](https://redirect.github.com/duckdb/duckdb-python/releases))\n| `==1.4.3` → `==1.4.4` |\n![age](https://developer.mend.io/api/mc/badges/age/pypi/duckdb/1.4.4?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/pypi/duckdb/1.4.3/1.4.4?slim=true)\n|\n|\n[github.com/apache/arrow-go/v18](https://redirect.github.com/apache/arrow-go)\n| `v18.5.0` → `v18.5.1` |\n![age](https://developer.mend.io/api/mc/badges/age/go/github.com%2fapache%2farrow-go%2fv18/v18.5.1?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/go/github.com%2fapache%2farrow-go%2fv18/v18.5.0/v18.5.1?slim=true)\n|\n\n---\n\n### Release Notes\n\n<details>\n<summary>duckdb/duckdb-python (duckdb)</summary>\n\n###\n[`v1.4.4`](https://redirect.github.com/duckdb/duckdb-python/releases/tag/v1.4.4):\nBugfix Release\n\n[Compare\nSource](https://redirect.github.com/duckdb/duckdb-python/compare/v1.4.3...v1.4.4)\n\n**DuckDB core v1.4.4 Changelog**:\n<https://github.com/duckdb/duckdb/compare/v1.4.3...v1.4.4>\n\n#### What's Changed in the Python Extension\n\n- fix polars tests by\n[@&#8203;evertlammerts](https://redirect.github.com/evertlammerts) in\n[#&#8203;218](https://redirect.github.com/duckdb/duckdb-python/pull/218)\n- tests for string and binary views by\n[@&#8203;evertlammerts](https://redirect.github.com/evertlammerts) in\n[#&#8203;221](https://redirect.github.com/duckdb/duckdb-python/pull/221)\n- Quote view names in unregister by\n[@&#8203;evertlammerts](https://redirect.github.com/evertlammerts) in\n[#&#8203;222](https://redirect.github.com/duckdb/duckdb-python/pull/222)\n- Limit string nodes in Polars expressions to constant expressions by\n[@&#8203;evertlammerts](https://redirect.github.com/evertlammerts) in\n[#&#8203;225](https://redirect.github.com/duckdb/duckdb-python/pull/225)\n- Escape identifiers in relation aggregations by\n[@&#8203;evertlammerts](https://redirect.github.com/evertlammerts) in\n[#&#8203;272](https://redirect.github.com/duckdb/duckdb-python/pull/272)\n- Fix DECREF bug during interpreter shutdown by\n[@&#8203;evertlammerts](https://redirect.github.com/evertlammerts) in\n[#&#8203;275](https://redirect.github.com/duckdb/duckdb-python/pull/275)\n- Support for Pandas 3.0.0 by\n[@&#8203;evertlammerts](https://redirect.github.com/evertlammerts) in\n[#&#8203;277](https://redirect.github.com/duckdb/duckdb-python/pull/277)\n- Prepare for v1.4.4 by\n[@&#8203;evertlammerts](https://redirect.github.com/evertlammerts) in\n[#&#8203;280](https://redirect.github.com/duckdb/duckdb-python/pull/280)\n\n**Full Changelog**:\n<https://github.com/duckdb/duckdb-python/compare/v1.4.3...v1.4.4>\n\n</details>\n\n<details>\n<summary>apache/arrow-go (github.com/apache/arrow-go/v18)</summary>\n\n###\n[`v18.5.1`](https://redirect.github.com/apache/arrow-go/releases/tag/v18.5.1)\n\n[Compare\nSource](https://redirect.github.com/apache/arrow-go/compare/v18.5.0...v18.5.1)\n\n#### What's Changed\n\n- fix(internal): fix assertion on undefined behavior by\n[@&#8203;amoeba](https://redirect.github.com/amoeba) in\n[#&#8203;602](https://redirect.github.com/apache/arrow-go/pull/602)\n- chore: Bump actions/upload-artifact from 5.0.0 to 6.0.0 by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;611](https://redirect.github.com/apache/arrow-go/pull/611)\n- chore: Bump google.golang.org/protobuf from 1.36.10 to 1.36.11 by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;607](https://redirect.github.com/apache/arrow-go/pull/607)\n- chore: Bump github.com/pierrec/lz4/v4 from 4.1.22 to 4.1.23 by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;616](https://redirect.github.com/apache/arrow-go/pull/616)\n- chore: Bump golang.org/x/tools from 0.39.0 to 0.40.0 by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;609](https://redirect.github.com/apache/arrow-go/pull/609)\n- chore: Bump actions/cache from 4 to 5 by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;608](https://redirect.github.com/apache/arrow-go/pull/608)\n- chore: Bump actions/download-artifact from 6.0.0 to 7.0.0 by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;610](https://redirect.github.com/apache/arrow-go/pull/610)\n- ci(benchmark): switch to new conbench instance by\n[@&#8203;rok](https://redirect.github.com/rok) in\n[#&#8203;593](https://redirect.github.com/apache/arrow-go/pull/593)\n- fix(flight): make StreamChunksFromReader ctx aware and\ncancellation-safe by\n[@&#8203;arnoldwakim](https://redirect.github.com/arnoldwakim) in\n[#&#8203;615](https://redirect.github.com/apache/arrow-go/pull/615)\n- fix(parquet/variant): fix basic stringify by\n[@&#8203;zeroshade](https://redirect.github.com/zeroshade) in\n[#&#8203;624](https://redirect.github.com/apache/arrow-go/pull/624)\n- chore: Bump github.com/google/flatbuffers from 25.9.23+incompatible to\n25.12.19+incompatible by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;617](https://redirect.github.com/apache/arrow-go/pull/617)\n- chore: Bump google.golang.org/grpc from 1.77.0 to 1.78.0 by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;621](https://redirect.github.com/apache/arrow-go/pull/621)\n- chore: Bump golang.org/x/tools from 0.40.0 to 0.41.0 by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;626](https://redirect.github.com/apache/arrow-go/pull/626)\n- fix(parquet/pqarrow): fix partial struct panic by\n[@&#8203;zeroshade](https://redirect.github.com/zeroshade) in\n[#&#8203;630](https://redirect.github.com/apache/arrow-go/pull/630)\n- Flaky test fixes by\n[@&#8203;zeroshade](https://redirect.github.com/zeroshade) in\n[#&#8203;629](https://redirect.github.com/apache/arrow-go/pull/629)\n- ipc: clear variadicCounts in recordEncoder.reset() by\n[@&#8203;asubiotto](https://redirect.github.com/asubiotto) in\n[#&#8203;631](https://redirect.github.com/apache/arrow-go/pull/631)\n- fix(arrow/cdata): Handle errors to prevent panic by\n[@&#8203;xiaocai2333](https://redirect.github.com/xiaocai2333) in\n[#&#8203;614](https://redirect.github.com/apache/arrow-go/pull/614)\n- chore: Bump github.com/substrait-io/substrait-go/v7 from 7.2.0 to\n7.2.2 by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;612](https://redirect.github.com/apache/arrow-go/pull/612)\n- chore: bump version to 18.5.1 by\n[@&#8203;zeroshade](https://redirect.github.com/zeroshade) in\n[#&#8203;632](https://redirect.github.com/apache/arrow-go/pull/632)\n\n#### New Contributors\n\n- [@&#8203;rok](https://redirect.github.com/rok) made their first\ncontribution in\n[#&#8203;593](https://redirect.github.com/apache/arrow-go/pull/593)\n- [@&#8203;asubiotto](https://redirect.github.com/asubiotto) made their\nfirst contribution in\n[#&#8203;631](https://redirect.github.com/apache/arrow-go/pull/631)\n- [@&#8203;xiaocai2333](https://redirect.github.com/xiaocai2333) made\ntheir first contribution in\n[#&#8203;614](https://redirect.github.com/apache/arrow-go/pull/614)\n\n**Full Changelog**:\n<https://github.com/apache/arrow-go/compare/v18.5.0...v18.5.1>\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - \"before 8am every weekday\" (UTC),\nAutomerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n👻 **Immortal**: This PR will be recreated if closed unmerged. Get\n[config\nhelp](https://redirect.github.com/renovatebot/renovate/discussions) if\nthat's undesired.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0Mi45Mi4xIiwidXBkYXRlZEluVmVyIjoiNDIuOTIuMSIsInRhcmdldEJyYW5jaCI6Im1haW4iLCJsYWJlbHMiOlsiZGVwZW5kZW5jaWVzIl19-->\n\n---------\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: otelbot <197425009+otelbot@users.noreply.github.com>\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-01-27T17:02:49Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/e1c7a802b626d7c8a6061e9f1a3ced60ac9417eb"
        },
        "date": 1769563314480,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 42.50403103760082,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 43.362995017793594,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 36.129557291666664,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 38.6484375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106660.53901870004,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106660.53901870004,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003447,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2450806.245458832,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 538227.4636989771,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 41.64697995082403,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 43.038094508317215,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.73619791666667,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.3828125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106664.91025114454,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106664.91025114454,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000988,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2173548.7419576515,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2083423.37287287,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
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
          "id": "92fcfc3adeabafb0240b40613f18d6a87f8df833",
          "message": "Formatting and encoding for scope attributes (#1898)\n\n# Change Summary\n\nPart of https://github.com/open-telemetry/otel-arrow/issues/1576, part\nof #1903.\n\nHalf of #1895, for a reasonable sized PR.\n\nThis PR:\n\n- Refactors the self_tracing formatter to fix poor structure. A new type\nStyledBufWriter separates the behavior of formatting log messages (w/\ncolor option) from the behavior of ConsoleWriter.\n- Adds ScopeFormatter argument to the basic log format, which formats a\nsuffix. Different callers use this differently, e.g., raw_error! ignores\nit, console_direct/async will append a suffix, and console_exporter\nbypasses b/c scopes print on a separate line\n- Adds ScopeToBytesMap for caching pre-calculated OTLP scope attributes\nas Bytes (with benchmark) and add a use in ITR\n- Extends LogRecord with LogContext, defines LogContextFn to be\nconfigured later in #1895\n- Adds TODOs for console_direct, console_async, and ITS provider mode,\ncurrently using empty context\n\n## How are these changes tested?\n\nNew test for encoding and formatting a scope/entity key.\n\n## Are there any user-facing changes?\n\nNo. See #1895.\n\n---------\n\nCo-authored-by: Utkarsh Umesan Pillai <66651184+utpilla@users.noreply.github.com>",
          "timestamp": "2026-01-28T15:18:59Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/92fcfc3adeabafb0240b40613f18d6a87f8df833"
        },
        "date": 1769619381859,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.484375,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 41.61816976359579,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 42.586379918026445,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.44869791666667,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.1875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106663.58586676154,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108246.87346947129,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001733,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2194808.493498924,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2061685.368874952,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 42.584691412552836,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 43.531058539073975,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 38.04635416666667,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 40.4921875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106664.59381806014,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106664.59381806014,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001166,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2448615.6601015027,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 528415.9826375798,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Utkarsh Umesan Pillai",
            "username": "utpilla",
            "email": "66651184+utpilla@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "b1edd55a1fc81a7c05a3274650011e128da0b269",
          "message": "[otap-df-engine] Fix error kind (#1908)\n\n## Changes\n- Use the correct error kind",
          "timestamp": "2026-01-29T01:01:39Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/b1edd55a1fc81a7c05a3274650011e128da0b269"
        },
        "date": 1769649949501,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.578125,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 42.500662632039564,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 43.83627715522573,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 37.063151041666664,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 38.58203125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106665.00980351439,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 107281.66689144095,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000932,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2439943.974337678,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 535680.0172022576,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 41.05882604123334,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 41.7568874912865,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 37.239322916666666,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 39.0390625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106665.38134882141,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106665.38134882141,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000723,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2182165.00053486,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2066302.1225176628,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
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
          "id": "e18aa77064e45cdcfe526303105e59a469dc63ee",
          "message": "chore(deps): update dependency psutil to v7.2.2 (#1910)\n\nThis PR contains the following updates:\n\n| Package | Change |\n[Age](https://docs.renovatebot.com/merge-confidence/) |\n[Confidence](https://docs.renovatebot.com/merge-confidence/) |\n|---|---|---|---|\n| [psutil](https://redirect.github.com/giampaolo/psutil) | `==7.2.1` →\n`==7.2.2` |\n![age](https://developer.mend.io/api/mc/badges/age/pypi/psutil/7.2.2?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/pypi/psutil/7.2.1/7.2.2?slim=true)\n|\n\n---\n\n### Release Notes\n\n<details>\n<summary>giampaolo/psutil (psutil)</summary>\n\n###\n[`v7.2.2`](https://redirect.github.com/giampaolo/psutil/blob/HEAD/HISTORY.rst#722)\n\n[Compare\nSource](https://redirect.github.com/giampaolo/psutil/compare/release-7.2.1...release-7.2.2)\n\n\\=====\n\n2026-01-28\n\n**Enhancements**\n\n- 2705\\_: \\[Linux]: `Process.wait()`\\_ now uses `pidfd_open()` +\n`poll()` for\n  waiting, resulting in no busy loop and faster response times. Requires\n  Linux >= 5.3 and Python >= 3.9. Falls back to traditional polling if\n  unavailable.\n- 2705\\_: \\[macOS], \\[BSD]: `Process.wait()`\\_ now uses `kqueue()` for\nwaiting,\n  resulting in no busy loop and faster response times.\n\n**Bug fixes**\n\n- 2701\\_, \\[macOS]: fix compilation error on macOS < 10.7. (patch by\nSergey\n  Fedorov)\n- 2707\\_, \\[macOS]: fix potential memory leaks in error paths of\n  `Process.memory_full_info()` and `Process.threads()`.\n- 2708\\_, \\[macOS]: Process.cmdline()`_ and `Process.environ()`_ may\nfail with ``OSError: [Errno 0] Undefined error`` (from\n``sysctl(KERN_PROCARGS2)``).\n  They now raise `AccessDenied\\`\\_ instead.\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - \"before 8am every weekday\" (UTC),\nAutomerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0Mi45Mi4xIiwidXBkYXRlZEluVmVyIjoiNDIuOTIuMSIsInRhcmdldEJyYW5jaCI6Im1haW4iLCJsYWJlbHMiOlsiZGVwZW5kZW5jaWVzIl19-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-01-29T01:16:36Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/e18aa77064e45cdcfe526303105e59a469dc63ee"
        },
        "date": 1769706758186,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.5625,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 40.92344966766992,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 41.7282637335399,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.09622395833333,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.1796875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106665.35823827227,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108332.00446074527,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000736,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2130527.6378070093,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2015667.9829780872,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -3.047619104385376,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 42.31232042584796,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 43.21452420091324,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 37.762109375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 40.6015625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104998.68751640605,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108198.64751690604,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00075,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2476138.9857141995,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 538148.2209541186,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
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
          "id": "2d1f9b0bd4eefcc144e4a89c69729921df7c0be3",
          "message": "fix: Batches may differ by field order after unification (#1922)\n\n# Change Summary\n\nNote this is a band-aid to avoid larger changes, but it does solve a\nbunch of panics.\n\n- Project batches to the merged schema before coalescing (reorder the\nfields to be the same)\n\n## What issue does this PR close?\n\nRelated to: https://github.com/open-telemetry/otel-arrow/issues/1334.\n\n## How are these changes tested?\n\nNew unit tests for the coalescing.\n\n## Are there any user-facing changes?\n\nNo.\n\n---------\n\nCo-authored-by: Joshua MacDonald <jmacd@users.noreply.github.com>",
          "timestamp": "2026-01-30T00:26:59Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/2d1f9b0bd4eefcc144e4a89c69729921df7c0be3"
        },
        "date": 1769737558508,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 41.96805177162626,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 43.085530468098256,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 38.940755208333336,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 40.79296875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106665.02224757368,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106665.02224757368,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000925,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2190040.163637885,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2048495.0899926245,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 43.088314259133746,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 44.28778628966577,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 38.02734375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 40.7421875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106663.86851784922,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106663.86851784922,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001574,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2466872.5510469093,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 530476.5085594463,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
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
          "id": "6ad291b19e1b329ce9441810ea2b4a41cd1085eb",
          "message": "Allow mixed local/shared pdata senders (#1919)\n\n# Change Summary\n\n- Allow local receivers/processors to use the generic message::Sender so\nmixed local/shared edges can share channels safely.\n- Introduce ChannelMode to centralize control-channel wiring and\nmetrics, reducing duplication across wrappers making the overall design\nless error-prone.\n- Add pipeline test for mixed local/shared receivers targeting the same\nexporter.\n  \n  ## What issue does this PR close?\n\n  NA\n  \n  ## How are these changes tested?\n\n See pipeline_tests.rs\n\n  ## Are there any user-facing changes?\n\n  No",
          "timestamp": "2026-01-30T03:15:37Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/6ad291b19e1b329ce9441810ea2b4a41cd1085eb"
        },
        "date": 1769792198699,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.6984126567840576,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 43.5346577706509,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 44.780785131263066,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 37.922916666666666,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 38.6875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104989.52904430332,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 105722.78924715242,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.005984,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2456153.681409964,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 532945.0156796998,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 42.253848429587514,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 43.14665042359249,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.667578125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.20703125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106659.85465728254,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106659.85465728254,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003832,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2186527.6134754936,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2093171.1597270563,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
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
          "id": "6ad291b19e1b329ce9441810ea2b4a41cd1085eb",
          "message": "Allow mixed local/shared pdata senders (#1919)\n\n# Change Summary\n\n- Allow local receivers/processors to use the generic message::Sender so\nmixed local/shared edges can share channels safely.\n- Introduce ChannelMode to centralize control-channel wiring and\nmetrics, reducing duplication across wrappers making the overall design\nless error-prone.\n- Add pipeline test for mixed local/shared receivers targeting the same\nexporter.\n  \n  ## What issue does this PR close?\n\n  NA\n  \n  ## How are these changes tested?\n\n See pipeline_tests.rs\n\n  ## Are there any user-facing changes?\n\n  No",
          "timestamp": "2026-01-30T03:15:37Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/6ad291b19e1b329ce9441810ea2b4a41cd1085eb"
        },
        "date": 1769822899875,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.5625,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 41.625087234459535,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 42.729756834244355,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.45260416666667,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.44921875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106664.79825495057,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108331.43572768418,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001051,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2176393.751985948,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2059556.4235922345,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 42.20206415472479,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 43.377094030635924,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 36.35364583333333,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 38.19140625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106659.25207099353,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106659.25207099353,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004171,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2441965.512914853,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 526647.9216028533,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          }
        ]
      }
    ]
  }
}