window.BENCHMARK_DATA = {
  "lastUpdate": 1784254428151,
  "repoUrl": "https://github.com/open-telemetry/otel-arrow",
  "entries": {
    "Benchmark": [
      {
        "commit": {
          "author": {
            "email": "33842784+JakeDern@users.noreply.github.com",
            "name": "Jake Dern",
            "username": "JakeDern"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "db316a8aff1b939ad6fe495fca7d948f7e9d1218",
          "message": "feat(comparison_dashboard): Run validate command on PRs (#2944)\n\n# Change Summary\n\nAdds a workflow for running some comparison dashboard validations.\nTriggered on PR, but I think still needs to be added as required in the\nrepo settings to actually block stuff unless we'd rather leave it as an\noptional check.\n\n## What issue does this PR close?\n\n* Closes #2869 \n\n## How are these changes tested?\n\nLoosely\n\n## Are there any user-facing changes?\n\nYes, new CI check.\n\n---------\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2026-05-12T23:56:23Z",
          "tree_id": "f5abe104ab8ca7f109e8f2b265d2ce30212c97b3",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/db316a8aff1b939ad6fe495fca7d948f7e9d1218"
        },
        "date": 1778636463909,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1129.310302734375,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 5.7882575703090335,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 6.286868937689144,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.026692708333332,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.8359375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 494.87951518605684,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 6083.605074614802,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.006525,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 214619.1557834158,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 176022.9862742083,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "cithomas@microsoft.com",
            "name": "Cijo Thomas",
            "username": "cijothomas"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "0c37386ba3e4cc9957ddb5a2b6a95fd6e2fa1752",
          "message": "fix(engine): centralize telemetry timer management in runtime manager (#2804)\n\nFixes https://github.com/open-telemetry/otel-arrow/issues/1305\n\n## Motivation\n\nOur idle perf tests were showing surprisingly high CPU usage.\nInvestigation revealed that each node was independently calling\n`start_periodic_telemetry(Duration::from_secs(1))`, triggering internal\nmetric collection (syscalls like `getrusage`, jemalloc stats, tokio\nworker metrics, channel snapshots) every second across all nodes. At 1s\nintervals, the telemetry overhead itself dominated idle CPU\nmeasurements, producing misleading results.\n\n## Changes\n\nCentralizes telemetry timer registration in the runtime control manager\nso all nodes use the configured `engine.telemetry.reporting_interval`\ninstead of each node managing its own timer independently. This removes\n~250 lines of per-node boilerplate across 15 nodes, with shutdown\ncancellation handled centrally.\n\nAlso bumps the idle perf test to `reporting_interval: 5s` (with matching\n5s Prometheus scrape) for a more realistic deployment baseline. With\ncentralized timing and the 5s interval, idle CPU numbers look\nsignificantly better — engine CPU drops ~2.3x and pipeline CPU drops\n~6.4x compared to the previous 1s configuration.\n\nAdds validation rejecting zero-duration `reporting_interval` to prevent\naccidental spin loops.\n\n**Notes**\n- `perf_exporter.config.frequency` is now silently ignored;\ndeprecation/cleanup is a follow-up.\n- `start_periodic_telemetry` API is still public — TBD whether to\nremove.\n- TODO comment added in `pipeline_ctrl.rs` for an\neager-timer-registration race flagged in review (telemetry can queue\nahead of `Shutdown` in a slow-starting node's bounded control channel);\nto be addressed via a node-ready signal in a follow-up.\n\n---------\n\nCo-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2026-05-13T00:09:18Z",
          "tree_id": "9f4b8b786a51649aafd8528ac6f7e3055f956f5d",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/0c37386ba3e4cc9957ddb5a2b6a95fd6e2fa1752"
        },
        "date": 1778643207472,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1080,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 5.775409019288323,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 6.208669507942658,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.571354166666666,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.6875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 511.9739149290344,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 6041.292196162605,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003057,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 215054.38206049547,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 176416.5105194937,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "33842784+JakeDern@users.noreply.github.com",
            "name": "Jake Dern",
            "username": "JakeDern"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "f1a007ec383bd70570a1d55f97fb970bd4baf08f",
          "message": "feat(comparison_dashboard): Add DFE metrics and traces baseline suites (#2950)\n\n# Change Summary\n\nThis PR adds metrics and traces baseline DFE suites\n\n## What issue does this PR close?\n\n* Closes #2947 \n\n## How are these changes tested?\n\n100k smoke test for every added suite.\n\n## Are there any user-facing changes?\n\nNo.",
          "timestamp": "2026-05-13T00:40:58Z",
          "tree_id": "7dae2eeb401a78dfc88d5125fbbe641902af2f29",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/f1a007ec383bd70570a1d55f97fb970bd4baf08f"
        },
        "date": 1778644245580,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1079.6610107421875,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 5.763688497232001,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 6.238613861386138,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.322005208333334,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.73828125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 503.32043530286336,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5937.474965606659,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.017432,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 213615.74014503317,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 174871.6880136071,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "cithomas@microsoft.com",
            "name": "Cijo Thomas",
            "username": "cijothomas"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "b645a269cc18cba6310ca15648a990f4bdc94e68",
          "message": "fix: restore uncapped throughput in traffic generator (#2946)\n\nPR #2723 broke uncapped mode — saturation tests dropped from ~290K to\n~1.5K logs/sec. This restores the original behavior.",
          "timestamp": "2026-05-13T00:41:34Z",
          "tree_id": "0dcd4ec18b4478c57fde24dcce2c41b41daee571",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/b645a269cc18cba6310ca15648a990f4bdc94e68"
        },
        "date": 1778645988531,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1136.151611328125,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 95.91028054800113,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.16134432861365,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 32.581380208333336,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 33.79296875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 43900.95107894757,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 542682.310713521,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004167,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 15386134.543090884,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 15342412.473993365,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "lalit_fin@yahoo.com",
            "name": "Lalit Kumar Bhasin",
            "username": "lalitb"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "4fced8c19ccd3ed3cb67d357c4ffa3d0b859733f",
          "message": "Add Linux user_events receiver (#2787)\n\n# Change Summary\n\nSplit out from the original combined PR #2717.\n\n  Adds a contrib receiver for Linux user_events tracepoints.\n\n  The receiver supports two decode formats:\n- `tracefs`: generic Linux tracepoint decoding based on the static\nschema exposed in tracefs format files.\n- `event_header`: decodes self-describing EventHeader payloads.\nEventHeader comes from the Microsoft LinuxTracepoints-Rust project, but\nis open and usable by any producer.\n\nThe receiver uses `one_collect` for `perf`/`user_events` collection, and\nsupports single or multiple subscriptions, batching, late registration,\nand basic receiver metrics. It is behind the `userevents-receiver`\nfeature flag.\n\nAlso adds a Linux CI smoke test that registers and reads both tracefs\nand EventHeader user_events tracepoints when kernel support is\navailable.\n\n\n## What issue does this PR close?\n\n* Closes #2698\n\n## How are these changes tested?\n\nAdded Linux CI smoke test\n\n## Are there any user-facing changes?\n\nYes, a receiver.\n\n---------\n\nCo-authored-by: Utkarsh Umesan Pillai <66651184+utpilla@users.noreply.github.com>",
          "timestamp": "2026-05-13T17:49:35Z",
          "tree_id": "7f8242c1c6e4f2f7eac4254410c53d7a7c6272f1",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/4fced8c19ccd3ed3cb67d357c4ffa3d0b859733f"
        },
        "date": 1778697787064,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1100.4676513671875,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.80296270799728,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.38197217928904,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 32.24296875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 32.90625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 45605.83548849392,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 547483.336598543,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.006356,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 15552943.748627268,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 15522105.706624521,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "29139614+renovate[bot]@users.noreply.github.com",
            "name": "renovate[bot]",
            "username": "renovate[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "cf292e5254e092ed378f23502d44a3be975ff67b",
          "message": "chore(deps): update geneva-uploader digest to 24b5ae5 (#2915)\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n|---|---|---|---|\n| geneva-uploader | workspace.dependencies | digest | `ce866b4` →\n`24b5ae5` |\n\n---\n\n### Configuration\n\n📅 **Schedule**: (UTC)\n\n- Branch creation\n  - At any time (no schedule defined)\n- Automerge\n  - At any time (no schedule defined)\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My4xNTkuMiIsInVwZGF0ZWRJblZlciI6IjQzLjE1OS4yIiwidGFyZ2V0QnJhbmNoIjoibWFpbiIsImxhYmVscyI6WyJkZXBlbmRlbmNpZXMiXX0=-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: Lalit Kumar Bhasin <lalit_fin@yahoo.com>\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-05-13T22:48:27Z",
          "tree_id": "f78add47efb80ec472665bff14510068d7ba5c6d",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/cf292e5254e092ed378f23502d44a3be975ff67b"
        },
        "date": 1778716292941,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1121.0526123046875,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.28233957505603,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.40350893478764,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 32.30260416666667,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 33.125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 44574.39682103506,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 544276.8453936913,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.016516,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 15461939.593305077,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 15412893.47028589,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "33842784+JakeDern@users.noreply.github.com",
            "name": "Jake Dern",
            "username": "JakeDern"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "158ad9d9ccf91af559e3972a250f94e0e17e8c26",
          "message": "feat(comparison_dashboard): Record, display, and validate environment details for suite runs (#2958)\n\n# Change Summary\n\nThis PR adds environment details like cpu architecture/model, available\ncore counts/memory, os, and run start/end times.\n\nWe also validate at dashboard build time that the environment for all\ntests within a comparison was the same and fail the build if not.\n\n## What issue does this PR close?\n\n* Closes #2949\n\n## How are these changes tested?\n\n<img width=\"2397\" height=\"1960\" alt=\"image\"\nsrc=\"https://github.com/user-attachments/assets/df37ed3b-364d-4ee1-ab80-87a705bd43dd\"\n/>\n\n## Are there any user-facing changes?\n\nYes, new information in the dashboard.\n\n---------\n\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-05-13T23:03:36Z",
          "tree_id": "5fb9153454db354c5f277288d1d3f471e9cc1355",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/158ad9d9ccf91af559e3972a250f94e0e17e8c26"
        },
        "date": 1778717331598,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1116.6824951171875,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.15369945748226,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.2371627690878,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 32.721614583333334,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 33.58203125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 44752.692479123245,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 544498.2022030982,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.006222,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 15510154.618027855,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 15460977.95559286,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "33842784+JakeDern@users.noreply.github.com",
            "name": "Jake Dern",
            "username": "JakeDern"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "55099cb930c81eb11c892e263d75ffcdb071ed08",
          "message": "feat(comparison_dashboard): Add OTC metrics and traces baselines (#2959)\n\n# Change Summary\n\nAdds the OTC metrics and traces baselines to mirror the logs ones.\n\n## What issue does this PR close?\n\n* Closes #2948\n\n## How are these changes tested?\n\nSmoke tested the runs locally.\n\n## Are there any user-facing changes?\n\nNew suites, but not really as they're not visible yet.",
          "timestamp": "2026-05-13T23:04:26Z",
          "tree_id": "cb9fa2580392e6ead6a893b47dda12fa66ea612b",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/55099cb930c81eb11c892e263d75ffcdb071ed08"
        },
        "date": 1778718373193,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1099.518798828125,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 95.9174769946192,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 99.78688961139495,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 31.985416666666666,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 32.88671875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 44326.038289501936,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 531699.1484139198,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.006265,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 15224831.65927939,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 15181203.147154313,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "66651184+utpilla@users.noreply.github.com",
            "name": "Utkarsh Umesan Pillai",
            "username": "utpilla"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "f0b1ca6c8504355628090d13bcb6490ba7a80664",
          "message": "Remove stale TODOs from OTLP Http Exporter that are already implemented (#2969)\n\n# Change Summary\n\nThe module-level `ToDo:` block at the top of\n`crates/core-nodes/src/exporters/otlp_http_exporter/mod.rs` still listed\ntwo items that have since been implemented. This PR removes them so the\nlist reflects only work that is actually outstanding.\n\n## Removed entries and where they are implemented\n\n### 1. `TLS/mTLS`\n\n- Configured via `HttpClientSettings.tls: Option<TlsClientConfig>` in\n`crates/otap/src/otlp_http/client_settings.rs` (lines 51-53).\n- Trust store (CA file/PEM, system roots), `insecure_skip_verify`, and\nthe mTLS client identity (cert + key paired into a reqwest `Identity`)\nare wired into the `reqwest::ClientBuilder` at\n`crates/otap/src/otlp_http/client_settings.rs` (lines 85-188).\n- Exporter validates and surfaces TLS-specific configuration errors at\n`crates/core-nodes/src/exporters/otlp_http_exporter/mod.rs` (lines\n139-175), e.g. unsupported `server_name_override`, ignored `insecure:\ntrue` when scheme is `https://`.\n- Integration coverage in `crates/otap/tests/otlp_exporter_tls.rs`:\n  - `otlp_exporter_connects_with_mtls`\n  - `otlp_exporter_connects_with_tls_only`\n  - `otlp_exporter_fails_partial_mtls`\n  - `otlp_exporter_fails_with_invalid_ca_pem`\n  - `otlp_exporter_allows_http_with_tls_config`\n- Proxy + TLS combinations in\n`crates/otap/tests/otlp_exporter_proxy_tls.rs`.\n\n### 2. `Allow endpoint overrides for each signal type (similar to Go\ncollector implementation)`\n\n- Per-signal override fields exist on the config struct:\n`traces_endpoint`, `metrics_endpoint`, `logs_endpoint` in\n`crates/core-nodes/src/exporters/otlp_http_exporter/config.rs` (lines\n26-40).\n- Resolved per-signal at construction time at\n`crates/core-nodes/src/exporters/otlp_http_exporter/mod.rs` (lines\n156-167 and 203-221), then dispatched per `SignalType` in the export\npath at lines 368-370 of the same file.\n\n## Remaining ToDo items (unchanged)\n\n- Proxy settings\n- Compression (payloads and accepting compressed responses)\n- JSON encoding payloads (only proto is supported, not configurable; see\nexisting inline TODO at\n`crates/core-nodes/src/exporters/otlp_http_exporter/mod.rs` line 670)\n- Unit test metrics reporting\n\n## What issue does this PR close?\n\n* Closes #NNN\n\n## How are these changes tested?\n\n## Are there any user-facing changes?\n\n <!-- If yes, provide further info below -->\n\nCo-authored-by: Utkarsh Umesan Pillai <utpilla@users.noreply.github.com>\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-05-14T00:30:31Z",
          "tree_id": "32472846cecce6c9b48427b60efe12b74a8eedff",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/f0b1ca6c8504355628090d13bcb6490ba7a80664"
        },
        "date": 1778721388859,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1120.076416015625,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.15608385046556,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.14895527008203,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 31.828645833333333,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 32.765625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 44626.39171034643,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 544476.1099497525,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003955,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 15471626.692371638,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 15422949.64242879,
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
          "id": "dbd487cad167f94df0ea1f00758212aeb9293163",
          "message": "feat: add num_connections config to OTLP gRPC exporter (#2967)\n\n## Summary\n\nAdd a `num_connections` configuration option to the OTLP gRPC exporter\nthat controls how many independent TCP connections (tonic Channels) are\ncreated per pipeline.\n\nFixes https://github.com/open-telemetry/otel-arrow/issues/1323\n\n## Problem\n\nWhen the receiver uses `SO_REUSEPORT` across multiple cores, the kernel\ndistributes **new TCP connections** (not individual RPCs) across\nlistener sockets. With the previous behavior of 1 gRPC channel per\npipeline, this caused severe core imbalance — e.g., with 2 engine cores:\none core at 60% and another at 94%.\n\n## Solution\n\n- Added `num_connections` config field (default: 1) to the OTLP gRPC\nexporter\n- When `num_connections > 1`, creates N independent tonic Channels, each\nestablishing its own TCP connection\n- Rewrote `GrpcClientPool` to use a FIFO `VecDeque` for round-robin\ndistribution of gRPC clients across channels\n- Pool is sized to `max(max_in_flight, num_connections)` ensuring every\nchannel gets at least one client\n- Updated saturation test templates to set `num_connections = num_cores\n* 4`\n\n## Results\n\nWith `num_connections` set appropriately:\n- Core imbalance fixed: 60%/94% → 99%/99%\n- 2-core throughput improved from 0.90× to 1.36× of 1-core baseline\n\n| Config | logs/sec | Scaling | Core balance |\n|--------|----------|---------|--------------|\n| 1-core, 1 conn (old) | 164,727 | baseline | N/A |\n| 2-core, 1 conn (old) | 148,685 | 0.90× | 60%/94% |\n| 1-core, 4 conns (new) | 177,461 | baseline | 99.6% |\n| 2-core, 8 conns (new) | 241,964 | 1.36× | 95.4% avg, balanced |\n\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-05-14T00:38:08Z",
          "tree_id": "a4279e76f65c3be4922d35583f6f3ef2275e5823",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/dbd487cad167f94df0ea1f00758212aeb9293163"
        },
        "date": 1778727812827,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1120.801513671875,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.25005357396212,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.17883809081766,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 32.733984375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 33.58984375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 44712.24475340919,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 545847.7665793103,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00325,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 15524991.625542825,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 15475085.804392053,
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
          "distinct": false,
          "id": "bdb39841a9ab8bb46062349d1b156ccb86d04482",
          "message": "fix(admin): group Prometheus metrics contiguously per spec (#2956)\n\n## Problem\n\nThe Prometheus exposition format requires all sample lines for a given\nmetric name to appear as one contiguous group, preceded by at most one\n`# HELP` and one `# TYPE` directive. The previous implementation\niterated per-entity, emitting each entity's full metric set inline. With\nN entities sharing the same metric set (e.g. 10 pipeline-thread cores),\nevery metric was re-opened N times non-contiguously. Strict parsers\n(like `prometheus_client.parser.text_string_to_metric_families`) split\nre-opened groups into separate unknown-type families, causing multi-core\nmetrics like `logs_produced` to report only one core's throughput.\n\n## Solution\n\nReplace the direct-emission approach with a **two-phase\ncollect-then-emit** pattern:\n\n1. **Collect phase**: During the registry visit, samples are grouped by\nmetric name into a `PromGroupedMetrics` structure that preserves\ninsertion order.\n2. **Emit phase**: After all entities are visited, each metric family is\nemitted as a contiguous block: `# HELP` → `# UNIT` → `# TYPE` → all\nsample lines.\n\nBoth `format_prometheus_text` and `agg_prometheus_text` are updated.\n\n## Testing\n\n- All 50 existing + new tests pass (`cargo xtask check` clean)\n- Added `test_format_prometheus_text_multi_entity_contiguous_grouping`\nthat registers two entities sharing a metric set and verifies:\n  - Exactly one `# HELP` and `# TYPE` per metric name\n- All lines (directives + samples) for each metric are contiguous (no\ngaps)\n  - Both entities' samples are present\n\nFixes #2945\n\nCo-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>",
          "timestamp": "2026-05-14T06:18:43Z",
          "tree_id": "5d32ce4a6f2c44c7628912de1830bfa9925102c9",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/bdb39841a9ab8bb46062349d1b156ccb86d04482"
        },
        "date": 1778742408680,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -2.0985770225524902,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.07148115975235,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.31899713688772,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 32.52200520833333,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 33.2734375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 530620.1288772116,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 541755.5999423361,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00285,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 15383539.635008264,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 15337969.33462937,
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
          "id": "77d6150a78e6580d2cbdc011304eabba39ffee68",
          "message": "feat(perf): add syslog TCP perf test through OTel Collector (#2962)\n\nAdd syslog TCP perf test suite routing traffic through the Go OTel\nCollector with syslog receiver, for comparison against the existing\ndf-engine syslog path.",
          "timestamp": "2026-05-14T06:25:15Z",
          "tree_id": "747800e13b64c3d60ae9e414886a100a91f16d7c",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/77d6150a78e6580d2cbdc011304eabba39ffee68"
        },
        "date": 1778743450894,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.3640714883804321,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.00441910272482,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.47395665634676,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 32.96302083333333,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 34.05078125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 534850.0876443463,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 542145.8251193928,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002159,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 15381890.889938988,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 15338070.399874859,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "33842784+JakeDern@users.noreply.github.com",
            "name": "Jake Dern",
            "username": "JakeDern"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "a47b96ab5d4676fdcc4717b3f68e817dea1804b3",
          "message": "fix(comparison_dashboard): Properly collect engine metrics for the go collector (#2971)\n\n# Change Summary\n\nUpdate the sql reports to grab metrics for OTC.\n\n## What issue does this PR close?\n\n* Closes #2855\n\n## How are these changes tested?\n\n<img width=\"2649\" height=\"1523\" alt=\"image\"\nsrc=\"https://github.com/user-attachments/assets/625e45a5-107c-4290-8ac0-f96470b6dbce\"\n/>\n\n## Are there any user-facing changes?\n\nNo.",
          "timestamp": "2026-05-14T14:59:00Z",
          "tree_id": "7d8b2cf2091bba2ededfa610e4da2794e09ea3a6",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/a47b96ab5d4676fdcc4717b3f68e817dea1804b3"
        },
        "date": 1778774578430,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.07829627394676208,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.17305881800974,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.11312480644163,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 31.838541666666668,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 32.5859375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 544862.4676505657,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 545289.0746562426,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.008391,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 15511411.003040247,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 15463061.97168377,
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
          "distinct": false,
          "id": "db9062212d3b6f4de5f72e1b1c875ed24b85f81b",
          "message": "Trim no-batch baselines from nightly batch processor perf suite (#2972)\n\n## Motivation\n\nThe Nightly Batch Processor perf suite\n(`tools/pipeline_perf_test/test_suites/integration/nightly/batch-processor-docker.yaml`)\ncurrently runs 12 tests = 3 signals (logs/metrics/traces) × 4 pipeline\nconfigs (OTAP+batch, OTAP-no-batch, OTLP+batch, OTLP-no-batch). It takes\n~24 min in nightly runs (e.g. [run\n25835389073](https://github.com/open-telemetry/otel-arrow/actions/runs/25835389073/job/75909238633)).\n\nThe no-batch baseline variants (`otap-otap`, `otlp-otlp`) don't actually\nexercise the batch processor, and equivalent passthrough OTAP/OTLP\ncoverage already exists in other nightly suites (`otelcol-docker.yaml`,\n`100klrps-batch-sizes-docker.yaml`).\n\n## Change\n\nRemove the 6 no-batch baseline tests from this suite, keeping only the 6\nbatch-enabled variants. Expected wall-time saving: roughly half (~12\nmin).\n\n## Remaining tests\n\n- Logs-OTAP-BATCH-OTAP, Logs-OTLP-BATCH-OTLP\n- Metrics-OTAP-BATCH-OTAP, Metrics-OTLP-BATCH-OTLP\n- Traces-OTAP-BATCH-OTAP, Traces-OTLP-BATCH-OTLP\n\nAll batch_processor regression coverage across (signal × transport) is\npreserved.",
          "timestamp": "2026-05-14T15:35:01Z",
          "tree_id": "f19aa07db63bfe8dcbbfeb350155d2ba00354498",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/db9062212d3b6f4de5f72e1b1c875ed24b85f81b"
        },
        "date": 1778783488907,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.04709576070308685,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.0957393810556,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 99.77360173428306,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 31.573046875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 32.234375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 543553.5479841867,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 543297.5573022475,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002184,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 15460964.452846544,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 15410933.122611875,
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
          "distinct": false,
          "id": "54598c7d91f10b5258113ceeb5f65baee0045cc9",
          "message": "Validate otel_* event names at compile time (#2957)\n\nAdds a `const fn validate_event_name` invoked from each `otel_info!` /\n`otel_warn!` / `otel_debug!` / `otel_error!` / `otel_event!` macro arm,\nso calls with empty or whitespace-containing event names fail `cargo\ncheck` instead of silently producing logs whose entire description is\nshoved into `event.name`.\n\nZero runtime cost (compile-time only). Existing call sites that violated\nthe rule are updated to use a short identifier with the descriptive text\nmoved to a `message = \"...\"` field.\n\nThe local macro copies in the `quiver` crate are intentionally left for\na follow-up.",
          "timestamp": "2026-05-14T15:40:17Z",
          "tree_id": "1eba98f7b02b4f1f65f1462a094f14855a2d3f95",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/54598c7d91f10b5258113ceeb5f65baee0045cc9"
        },
        "date": 1778785028671,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.0861191600561142,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.0003003291857,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.03549918799783,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 32.442708333333336,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 33.34375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 544952.1329815411,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 545421.4411677774,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003215,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 15487343.691512067,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 15437512.335388213,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "29139614+renovate[bot]@users.noreply.github.com",
            "name": "renovate[bot]",
            "username": "renovate[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "300c8733c5e7430472ace73b6e92cdccded66294",
          "message": "chore(deps): update pipeline perf python dependencies (#2931)\n\nThis PR contains the following updates:\n\n| Package | Change |\n[Age](https://docs.renovatebot.com/merge-confidence/) |\n[Confidence](https://docs.renovatebot.com/merge-confidence/) |\n|---|---|---|---|\n| [pandas](https://redirect.github.com/pandas-dev/pandas) | `==3.0.2` →\n`==3.0.3` |\n![age](https://developer.mend.io/api/mc/badges/age/pypi/pandas/3.0.3?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/pypi/pandas/3.0.2/3.0.3?slim=true)\n|\n| [requests](https://redirect.github.com/psf/requests)\n([changelog](https://redirect.github.com/psf/requests/blob/master/HISTORY.md))\n| `==2.33.1` → `==2.34.1` |\n![age](https://developer.mend.io/api/mc/badges/age/pypi/requests/2.34.1?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/pypi/requests/2.33.1/2.34.1?slim=true)\n|\n\n---\n\n### Release Notes\n\n<details>\n<summary>pandas-dev/pandas (pandas)</summary>\n\n###\n[`v3.0.3`](https://redirect.github.com/pandas-dev/pandas/releases/tag/v3.0.3):\npandas 3.0.3\n\n[Compare\nSource](https://redirect.github.com/pandas-dev/pandas/compare/v3.0.2...v3.0.3)\n\nWe are pleased to announce the release of pandas 3.0.3.\nThis is a patch release in the 3.0.x series and includes some regression\nfixes and bug fixes. We recommend that all users of the 3.0.x series\nupgrade to this version.\n\nSee the [full\nwhatsnew](https://pandas.pydata.org/docs/whatsnew/v3.0.3.html) for a\nlist of all the changes.\n\nPandas 3.0 supports Python 3.11 and higher.\nThe release can be installed from PyPI:\n\n```\npython -m pip install --upgrade pandas==3.0.*\n```\n\nOr from conda-forge\n\n```\nconda install -c conda-forge pandas=3.0\n```\n\nPlease report any issues with the release on the [pandas issue\ntracker](https://redirect.github.com/pandas-dev/pandas/issues).\n\nThanks to all the contributors who made this release possible.\n\n</details>\n\n<details>\n<summary>psf/requests (requests)</summary>\n\n###\n[`v2.34.1`](https://redirect.github.com/psf/requests/blob/HEAD/HISTORY.md#2341-2026-05-13)\n\n[Compare\nSource](https://redirect.github.com/psf/requests/compare/v2.34.0...v2.34.1)\n\n**Bugfixes**\n\n- Widened `json` input type from `dict` and `list` to `Mapping`\nand `Sequence`.\n([#&#8203;7436](https://redirect.github.com/psf/requests/issues/7436))\n- Changed `headers` input type to MutableMapping and removed `None` from\n`Request.headers` typing to improve handling for users.\n([#&#8203;7431](https://redirect.github.com/psf/requests/issues/7431))\n- `Response.reason` moved from `str | None` to `str` to improve handling\nfor users.\n([#&#8203;7437](https://redirect.github.com/psf/requests/issues/7437))\n- Fixed a bug where some bodies with custom `__getattr__`\nimplementations\nweren't being properly detected as Iterables.\n([#&#8203;7433](https://redirect.github.com/psf/requests/issues/7433))\n\n###\n[`v2.34.0`](https://redirect.github.com/psf/requests/blob/HEAD/HISTORY.md#2340-2026-05-11)\n\n[Compare\nSource](https://redirect.github.com/psf/requests/compare/v2.33.1...v2.34.0)\n\n**Announcements**\n\n- Requests 2.34.0 introduces inline types, replacing those provided by\ntypeshed. Public API types should be fully compatible with mypy,\npyright,\nand ty. We believe types are comprehensive but if you find issues,\nplease\n  report them to the pinned tracking issue.\n\nSpecial thanks to\n[@&#8203;bastimeyer](https://redirect.github.com/bastimeyer),\n[@&#8203;cthoyt](https://redirect.github.com/cthoyt),\n[@&#8203;edgarrmondragon](https://redirect.github.com/edgarrmondragon),\nand [@&#8203;srittau](https://redirect.github.com/srittau) for\nhelping review and test the types ahead of the release.\n([#&#8203;7272](https://redirect.github.com/psf/requests/issues/7272))\n\n**Improvements**\n\n- Digest Auth hashing algorithms have added `usedforsecurity=False` to\nclarify\nsecurity considerations.\n([#&#8203;7310](https://redirect.github.com/psf/requests/issues/7310))\n- Requests added support for Python 3.15 based on beta1. Downstream\nprojects\nshould be able to start testing prior to its release in October.\n([#&#8203;7422](https://redirect.github.com/psf/requests/issues/7422))\n- Requests added support for Python 3.14t.\n([#&#8203;7419](https://redirect.github.com/psf/requests/issues/7419))\n\n**Bugfixes**\n\n- `Response.history` no longer contains a reference to itself,\npreventing\naccidental looping when traversing the history list.\n([#&#8203;7328](https://redirect.github.com/psf/requests/issues/7328))\n- Requests no longer performs greedy matching on no\\_proxy domains. The\n  proxy\\_bypass implementation has been updated with CPython's fix from\nbpo-39057.\n([#&#8203;7427](https://redirect.github.com/psf/requests/issues/7427))\n- Requests no longer incorrectly strips duplicate leading slashes in\n  URI paths. This should address user issues with specific presigned\nURLs. Note the full fix requires urllib3 2.7.0+.\n([#&#8203;7315](https://redirect.github.com/psf/requests/issues/7315))\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: (UTC)\n\n- Branch creation\n  - \"before 8am every weekday\"\n- Automerge\n  - At any time (no schedule defined)\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n👻 **Immortal**: This PR will be recreated if closed unmerged. Get\n[config\nhelp](https://redirect.github.com/renovatebot/renovate/discussions) if\nthat's undesired.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My4xNTkuMiIsInVwZGF0ZWRJblZlciI6IjQzLjE3My42IiwidGFyZ2V0QnJhbmNoIjoibWFpbiIsImxhYmVscyI6WyJkZXBlbmRlbmNpZXMiXX0=-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2026-05-14T16:02:07Z",
          "tree_id": "121dc728c32272834700c5e7ecb96eef320d787a",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/300c8733c5e7430472ace73b6e92cdccded66294"
        },
        "date": 1778788639786,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.7816817760467529,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.20371751219731,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.36092677256877,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 32.658203125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 33.20703125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 540343.5874188459,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 544567.354900446,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003303,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 15474664.033396948,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 15426755.49057537,
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
          "id": "40b4f4d1112b1bc55f08185aa778865b4a43bd66",
          "message": "Set cache-bin: false on Swatinem/rust-cache to fix broken cargo on macos-latest (#2978)\n\n## Problem\n\nCI `clippy (*, macos-latest)` (and other macOS rust steps) started\nfailing today across many PRs with:\n\n```\nerror: error: unexpected argument 'clippy' found\nUsage: rustup-init[EXE] [OPTIONS]\n```\n\n## Root cause\n\nGitHub rolled out a new macos-latest runner image today\n([actions/runner-images#14037](https://github.com/actions/runner-images/pull/14037))\nthat changed how the `rustc`/`cargo` rustup proxy binaries are set up.\nCombined with\n[Swatinem/rust-cache#325](https://github.com/Swatinem/rust-cache/pull/325)\n(which made `cache-bin: true` the default in v2.8+), the cached\n`$CARGO_HOME/bin/` from previous runs gets restored over the\nfreshly-installed proxies, leaving `cargo` dispatching to `rustup-init`\nbehavior instead of the real cargo.\n\nTracked upstream:\n[Swatinem/rust-cache#341](https://github.com/Swatinem/rust-cache/issues/341).\n\n## Fix\n\nSet `cache-bin: false` on all 7 `Swatinem/rust-cache` invocations in\n`.github/workflows/rust-ci.yml`. This is the workaround confirmed by the\nupstream issue reporter. We don't `cargo install` any binaries that need\ncaching, so this loses no useful caching.",
          "timestamp": "2026-05-14T22:42:42Z",
          "tree_id": "165f39943b3c5c23dbec8d35f8c4e83ee60aa4b2",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/40b4f4d1112b1bc55f08185aa778865b4a43bd66"
        },
        "date": 1778801636860,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.49473848938941956,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.0284175809398,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.1754407670894,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 32.862239583333334,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 33.60546875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 543298.0281434,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 545985.9326331938,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002132,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 15548104.016322846,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 15498965.21565295,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "truffleagent@gmail.com",
            "name": "Truffle",
            "username": "truffle-dev"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "1bcb61866fbdc9b28420f409eb3de446fd8fcc02",
          "message": "Add OPL query-engine starts_with and ends_with functions (#2825)\n\nCloses #2819\n\nWires the upstream datafusion `starts_with` and `ends_with` UDFs into\nthe OPL query engine via the existing `InvokeFunctionExpr` path. Each\nfunction adds:\n\n- A function-name constant in `consts.rs`\n- A parser registration with two parameter placeholders in\n`parser.rs::default_parser_options`\n- A `from_func_name` arm in `DataFusionFunctionDef` (`expr.rs`)\nreturning `ExprLogicalType::Boolean` with `requires_dict_downcast:\ntrue`, matching the sha256 wiring\n\nExample queries that now work:\n\n```\nlogs | where starts_with(attributes[\"x\"], \"prefix\")\nlogs | where ends_with(event_name, \"suffix\")\n```\n\n## Tests\n\n- Unit tests in `expr.rs` build the `InvokeFunctionScalarExpression`\ndirectly, plan, execute against a `Logs` record batch, and assert a\n`BooleanArray` result. Patterned on `test_function_invocation_sha256`.\n- End-to-end OPL filter tests in `filter.rs` cover `event_name` and\n`attributes[\"...\"]` arguments, with the column on either side of the\npredicate.\n\n## Validation\n\n- `cargo check -p otap-df-query-engine`: clean\n- `cargo test -p otap-df-query-engine`: 548 passed (4 new filter tests,\n2 new expr tests)\n- `cargo clippy -p otap-df-query-engine --all-targets -- -D warnings`:\nclean\n- `cargo fmt --all -- --check`: clean\n- `cargo xtask quick-check`: clean\n\n## Notes\n\n`body` field tests are intentionally omitted because OTLP `body` is\nheterogeneous (`AnyValue` with string + int variants). The upstream\ndatafusion UDFs reject mixed types directly. `contains` works there\nbecause it has a custom string-coercing wrapper UDF; aligning\n`starts_with`/`ends_with` to that wrapper pattern is a follow-up beyond\nthe scope of #2819, which asks specifically for the upstream UDFs.\n\nSigned-off-by: truffle <truffleagent@gmail.com>\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-05-15T16:57:36Z",
          "tree_id": "a7924b87bf245f3e42bde72e5fb6737a6ab59980",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/1bcb61866fbdc9b28420f409eb3de446fd8fcc02"
        },
        "date": 1778868177167,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.6279927492141724,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.28495083889307,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.32110310635991,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 31.995182291666666,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 32.796875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 543499.8411673418,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 546912.9809187655,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003403,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 15538500.265274683,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 15489030.27095237,
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
          "id": "672d665198917e2ede1ed856a9f88925ef2b151f",
          "message": "perf: add egress_bytes_per_log metric to benchmark reports (#2982)\n\n## Summary\n\nTwo changes to benchmark report metrics:\n\n### 1. Add `egress_bytes_per_log` metric\nAdds a derived metric (bytes/log) computed as `network_tx_bytes_rate /\nlogs_received_rate`. This makes it easy to assess whether compression\nratios in tests are realistic.\n\nFor a ~150 byte log record, realistic values should be ~35-50 bytes/log.\nValues like ~27 bytes/log indicate unrealistic compression from\nlow-entropy test data (e.g., replayed identical payloads).\n\n### 2. Switch network metrics from bytes/sec to MB/s\nReplaces `network_tx_bytes_rate_avg` and `network_rx_bytes_rate_avg`\n(bytes/sec) with `network_tx_mb_per_sec` and `network_rx_mb_per_sec`\n(MB/s) for readability. Raw bytes/sec values like 2,689,390 are hard to\ninterpret at a glance; 2.56 MB/s is immediately meaningful.\n\n### Files changed\n- `integration/configs/integration_report_logs.yaml`\n- `integration/configs/integration_report_metrics.yaml`\n- `integration/configs/integration_report_traces.yaml`\n- `comparison_dashboard/reports/report_logs.yaml`\n- `comparison_dashboard/reports/report_metrics.yaml`\n- `comparison_dashboard/reports/report_traces.yaml`\n\nRelates to #2540",
          "timestamp": "2026-05-15T22:28:24Z",
          "tree_id": "7618a842ff3874f6b9ef7e64a70aab82e18392e6",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/672d665198917e2ede1ed856a9f88925ef2b151f"
        },
        "date": 1778887051306,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.6440542936325073,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.23439127975621,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.32779284833539,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 32.80234375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 33.5078125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 534594.2483186683,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 543383.2678840032,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002142,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_mb_per_sec",
            "value": 14.715021930014245,
            "unit": "MB/s",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_mb_per_sec",
            "value": 14.673850711119632,
            "unit": "MB/s",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 28.39582988149801,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "52658021+Paramesh324@users.noreply.github.com",
            "name": "Paramesh324",
            "username": "Paramesh324"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "cf5c5783c4a9013cde8f9bd072b6df9b5c7b924e",
          "message": "Add coalesce support in OPL parser and OTAP query-engine planner (#2905)\n\n# Change Summary\n\nAdd support for `coalesce(...)` in OPL / OTAP query-engine by:\n- Parsing `coalesce` calls in the OPL parser into\n`ScalarExpression::Coalesce(CoalesceScalarExpression)`.\n- Planning `ScalarExpression::Coalesce` in the columnar query-engine\nexpression planner.\n- Lowering coalesce arguments to a `CASE/WHEN` logical expression\n(DataFusion-compatible behavior) to ensure correct execution in the\ncurrent planning/execution path.\n- Adding parser and planner/execution tests for valid usage and arity\nvalidation.\n\n## What issue does this PR close?\n\n* Closes #2823\n\n## How are these changes tested?\n\n- Added OPL parser tests for:\n  - valid `coalesce(...)` parsing\n  - invalid arity (fewer than 2 args)\n- Added query-engine expr planner/execution test for coalesce fallback\nbehavior.\n- Ran formatting and checks:\n  - `cargo fmt --all`\n  - `cargo test -p otap-df-opl --lib`\n  - `cargo test -p otap-df-query-engine --lib pipeline::expr::test`\n- `cargo clippy -p otap-df-opl -p otap-df-query-engine --all-targets --\n-D warnings`\n\n\n## Are there any user-facing changes?\n\n Yes.\nUsers can now use `coalesce(...)` in OPL expressions for OTAP\nquery-engine pipelines, e.g.:\n`coalesce(attributes[\"x\"], attributes[\"y\"], \"hello\")`\n\n---------\n\nSigned-off-by: Parameshwaran Krishnasamy <Parameshwaran.K@ibm.com>\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-05-18T13:59:55Z",
          "tree_id": "4a35a5e8c30142b698c7752cb6220d6581c162d4",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/cf5c5783c4a9013cde8f9bd072b6df9b5c7b924e"
        },
        "date": 1779118186841,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.3732337951660156,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.02272937829981,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.33478908188587,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 32.774609375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 33.72265625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 537494.8868548304,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 544875.9485650928,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002208,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_mb_per_sec",
            "value": 14.721821794183567,
            "unit": "MB/s",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_mb_per_sec",
            "value": 14.680436281673602,
            "unit": "MB/s",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 28.331125736620173,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "33842784+JakeDern@users.noreply.github.com",
            "name": "Jake Dern",
            "username": "JakeDern"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "d8673a4b5dbaa0f6b055a45221f0933023a01d36",
          "message": "task(comparison_dashboard): Add signals/sec metrics for traces and metrics (#3004)\n\n# Change Summary\n\nAdds produced and consumed metrics for the traces and metrics signals to\nthe dashboard.\n\n## What issue does this PR close?\n\n* Closes #2990\n\n## How are these changes tested?\n\n<img width=\"2259\" height=\"308\" alt=\"image\"\nsrc=\"https://github.com/user-attachments/assets/2559662a-193b-44a2-8d28-70bd8df225d8\"\n/>\n\n## Are there any user-facing changes?\n\nYes, we plot more metrics on the dashboard.",
          "timestamp": "2026-05-18T15:57:27Z",
          "tree_id": "a91b42fd4e9c3d54534f3a494c65d50b9f975eb6",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/d8673a4b5dbaa0f6b055a45221f0933023a01d36"
        },
        "date": 1779128302863,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.8817921280860901,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.06992712266222,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.41282755410752,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 32.829166666666666,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 33.4296875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 537064.1991049097,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 541799.9892304773,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002659,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_mb_per_sec",
            "value": 14.652476186056044,
            "unit": "MB/s",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_mb_per_sec",
            "value": 14.610508032019636,
            "unit": "MB/s",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 28.357761488869436,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
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
          "distinct": false,
          "id": "e68150189f986e183d3e3773d77f52835647be5f",
          "message": "NUMA-aware core assignments for saturation tests (#2997)\n\n## Summary\n\nImproves the accuracy and consistency of the saturation/scaling test\nsuite by eliminating hardware-level interference (NUMA cross-socket\ntraffic, HT sibling contention) and making the workload more realistic.\n\n## Problem\n\nThe saturation tests measure CPU scaling efficiency by running the SUT\nat 1/2/4/8/16 cores and checking for linear throughput growth. On our\n2-socket NUMA CI box (Intel Xeon 8358, 128 logical CPUs), results were\nunreliable:\n\n- **16-core showed ~48% efficiency** — far below expected for a\nshared-nothing architecture\n- **High run-to-run variance** made results hard to interpret\n\nRoot causes identified:\n1. **HT sibling contention** — SUT (cores 0-15) and backend (cores\n64-79) were placed on hyperthreaded siblings of the same physical cores,\ncausing L1/L2 thrashing\n2. **Cross-NUMA traffic** — components spanning both NUMA nodes incurred\n2x memory latency (node distance: 10 local vs 20 remote)\n3. **Unrealistic workload** — semantic_conventions data source with\n~300-byte logs and low entropy bodies gave artificially good\ncompression, not representative of real pipelines\n\n## Changes\n\n### 1. NUMA-aware core pinning\n\nPin SUT exclusively to NUMA node 0 physical cores (0-15), loadgen and\nbackend to NUMA node 1 (32-63, 96-127). This ensures:\n- No HT sibling sharing between components\n- All memory accesses are NUMA-local within each component\n- SUT gets dedicated L3 cache (no pollution from loadgen/backend)\n\n### 2. Static 1KB log bodies (saturation tests only)\n\nSwitch from `semantic_conventions` (~300 bytes) to `static` data source\nwith `log_body_size_bytes: 1024`. Larger payloads better exercise the\nserialization/compression/network path that dominates real workloads.\nOnly affects saturation tests — all other tests continue using\n`semantic_conventions`.\n\n### 3. Body pool entropy (512 unique bodies)\n\nGenerate 512 unique log bodies with sequence-prefixed cycling templates\ninstead of repeating the same ~50 bodies. This gives a realistic ~3:1\ncompression ratio rather than artificially high compression from\nduplicate data.\n\n### 4. Saturation tests in label-triggered workflow\n\nAdd saturation suite to `pipeline-perf-on-label.yaml` so scaling can be\nvalidated on PRs via the `pipelineperf` label without waiting for\nnightly runs. Also adds `labeled` event type to trigger on label\naddition.\n\n## Results (3 consecutive CI runs on bare metal)\n\n| Cores | Before | Run 0 | Run 1 | Run 2 |\n|-------|--------|-------|-------|-------|\n| 2 | 97% | 96% | 116%* | 99% |\n| 4 | 87% | 99.6% | 102% | 100% |\n| 8 | 82% | 100% | 95% | 91% |\n| 16 | 48% | 80% | 68% | 91% |\n| **Avg** | **79%** | **94%** | **96%** | **96%** |\n| **Speedup** | **7.7x** | **12.8x** | **11.0x** | **14.6x** |\n\n*\\* >100% is measurement noise (super-linear artifact from cache warming\neffects)*\n\nVerdict changed from 🟠 ACCEPTABLE to ✅ EXCELLENT across all runs.\n\n16-core variance (68-91%) is due to loadgen CPU saturation — with 48\nloadgen cores at 100%, small throughput fluctuations swing the\nefficiency metric. This is a measurement limitation, not a SUT issue.\n\n## Files changed\n\n- `saturation-{1,2,4,8,16}cores.yaml` — NUMA-aware core ranges\n- `saturation-cores-template.yaml.j2` — `data_source: static`,\n`log_body_size_bytes: 1024`\n- `configs/loadgen/config.yaml.j2` — template variables for data_source\nand log_body_size_bytes\n- `df-loadgen-steps-docker.yaml` — pass through new template variables\n- `static_signal.rs` — body pool entropy (512 unique bodies with cycling\ntemplates)\n- `pipeline-perf-on-label.yaml` — saturation step + labeled trigger",
          "timestamp": "2026-05-18T16:10:36Z",
          "tree_id": "138a4810673b4ba36eb05261229f3f3bdb389934",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/e68150189f986e183d3e3773d77f52835647be5f"
        },
        "date": 1779138580611,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.2629070281982422,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 95.99592719406081,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.24567821401078,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 32.79296875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 33.4375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 537143.6561463312,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 543927.2813510387,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003315,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_mb_per_sec",
            "value": 14.739360978352858,
            "unit": "MB/s",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_mb_per_sec",
            "value": 14.696708676044901,
            "unit": "MB/s",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 28.414350055853863,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "33842784+JakeDern@users.noreply.github.com",
            "name": "Jake Dern",
            "username": "JakeDern"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "9a664bf7189cac7a48a4928e6d0630221cee0306",
          "message": "feat(comparison_dashboard): Create logs, metrics, and traces DFE comparisons (#3006)\n\n# Change Summary\n\nUpdate the manifest to point to three new comparisons for the dataflow\nengine - One each for logs, metrics, and traces. This is in preparation\nto publish logs/metrics/traces results for the dataflow engine.\n\nNote that this PR does not publish data, we will do that separately.\n\n## What issue does this PR close?\n\n* Closes #2992\n\n## How are these changes tested?\n\n<img width=\"2469\" height=\"1872\" alt=\"image\"\nsrc=\"https://github.com/user-attachments/assets/82f36c20-d121-4365-9a89-7c39dc4b0317\"\n/>\n\n## Are there any user-facing changes?\n\nYes, new charts on the dashboard.",
          "timestamp": "2026-05-18T16:20:57Z",
          "tree_id": "96a79253316923be7ab459e2a1a3dc52ea7de4f3",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/9a664bf7189cac7a48a4928e6d0630221cee0306"
        },
        "date": 1779139625393,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.1798279881477356,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.1912096090803,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.04765374435623,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 33.09010416666667,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 33.71484375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 545674.690129825,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 544693.4142992555,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003516,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_mb_per_sec",
            "value": 14.77857445038862,
            "unit": "MB/s",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_mb_per_sec",
            "value": 14.730348108519184,
            "unit": "MB/s",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 28.449873040647628,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "33842784+JakeDern@users.noreply.github.com",
            "name": "Jake Dern",
            "username": "JakeDern"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "e4f99262e235502cf1095e331581a74a8c1f6691",
          "message": "feat(comparison_dashboard): Specify tests for comparison in the manifest and comparison metadata (#3008)\n\n# Change Summary\n\nUpdates the site so that the tests to be plotted for comparison are\nspecified via some metadata rather than be inferred/hardcoded into the\nsite.\n\nWaiting on #3006 for a rebase.\n\n## What issue does this PR close?\n\n* Closes #3005 \n\n## How are these changes tested?\n\n<img width=\"2347\" height=\"663\" alt=\"image\"\nsrc=\"https://github.com/user-attachments/assets/c7af24b6-101c-46f8-9dce-28ece817b6a1\"\n/>\n\n## Are there any user-facing changes?\n\nNo.",
          "timestamp": "2026-05-18T17:53:16Z",
          "tree_id": "26613c2b725848b197409feb6a20dec32ab8aa59",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/e4f99262e235502cf1095e331581a74a8c1f6691"
        },
        "date": 1779140664786,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.0702394247055054,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.16370630807684,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.07327102803738,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 32.5203125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 33.06640625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 538167.3125667473,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 543926.9912726702,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003347,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_mb_per_sec",
            "value": 14.763075302029995,
            "unit": "MB/s",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_mb_per_sec",
            "value": 14.717798992543699,
            "unit": "MB/s",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 28.460081401147434,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "33842784+JakeDern@users.noreply.github.com",
            "name": "Jake Dern",
            "username": "JakeDern"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "b0aa43e4707ef35c6943a3fb1f138405e1b7bdad",
          "message": "task(comparison_dashboard): Update color palette (#3015)\n\n# Change Summary\n\nAdds a new 20 color palette that is maybe nicer based on this:\n\n<img width=\"924\" height=\"250\" alt=\"image\"\nsrc=\"https://github.com/user-attachments/assets/00712d13-0634-451a-b086-38ba0541a7b6\"\n/>\n\n## What issue does this PR close?\n\n* Closes #3014\n\n## How are these changes tested?\n\nBefore: \n\n<img width=\"2266\" height=\"554\" alt=\"before\"\nsrc=\"https://github.com/user-attachments/assets/565d49d3-cf61-4003-90fe-ea0dd1798abf\"\n/>\n\nAfter:\n\n<img width=\"2356\" height=\"642\" alt=\"image\"\nsrc=\"https://github.com/user-attachments/assets/e33958ce-9148-4564-bbd8-f67d134f6995\"\n/>\n\n## Are there any user-facing changes?\n\nColor!",
          "timestamp": "2026-05-18T18:17:55Z",
          "tree_id": "3c68b9b58495d6a0523d869a415e2a451cb8dcc1",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/b0aa43e4707ef35c6943a3fb1f138405e1b7bdad"
        },
        "date": 1779141702578,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.066266417503357,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.09538118673139,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.10480720387902,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 33.53138020833333,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 34.26953125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 540172.4350160536,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 545932.1126660944,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003358,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_mb_per_sec",
            "value": 14.81505520885154,
            "unit": "MB/s",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_mb_per_sec",
            "value": 14.76828724102629,
            "unit": "MB/s",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 28.45539027702905,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
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
          "id": "235a7d04315f1f5aa1c156ac300156f0fd7e17f5",
          "message": "Document AI-assisted component development guidance (#2909)\n\n# Change Summary\n\nAdds AI-assisted development guidance for OTAP Dataflow Engine\ncontributors and maintainers.\n\nThis PR introduces a concise `docs/ai` entry point and documents the\nproject’s posture for responsible AI-assisted work: controlled,\nreviewable, evidence-based, and owned by engineers familiar with OTAP\nDataflow, Rust, and OpenTelemetry.\n\nIt also clarifies the current AI-assisted guidance set:\n- `AI-Assisted Component Development`: overview for choosing the right\napproach.\n- `Spec-Constrained Oracle Reimplementation`: for\ninteroperability-focused work where a reference implementation acts as\nan executable oracle.\n- `Reference-Informed OTAP-Native Capability Design`: for designing\nimproved OTAP-native capabilities from existing implementations,\nfeedback, and future direction.\n- `AI-Assisted Pull Request Review`: for human and agent reviewers,\nfocused on OTAP architectural invariants, thread-per-core runtime\nbehavior, bounded resources, backpressure, performance, correctness,\nsecurity, portability, and test intent.\n  \n## What issue does this PR close?\n\n* Closes\n[#2908](https://github.com/open-telemetry/otel-arrow/issues/2908)\n\n## How are these changes tested?\n\n- Ran `python3 tools/sanitycheck.py`\n\n## Are there any user-facing changes?\n\nYes. This is documentation-only, but contributor-facing. It adds and\nupdates guidance for engineers using AI-assisted workflows in OTAP\nDataflow Engine development.\n\n---------\n\nCo-authored-by: albertlockett <a.lockett@f5.com>\nCo-authored-by: Joshua MacDonald <jmacd@users.noreply.github.com>",
          "timestamp": "2026-05-18T18:37:07Z",
          "tree_id": "575f35e189ea92ef820f9a6f858eeaa669cc051b",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/235a7d04315f1f5aa1c156ac300156f0fd7e17f5"
        },
        "date": 1779142736279,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.6625651121139526,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.18573407881023,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.07644830797888,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 32.35989583333333,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 33.66015625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 540897.980162101,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 544481.78126114,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00333,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_mb_per_sec",
            "value": 14.754023457155443,
            "unit": "MB/s",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_mb_per_sec",
            "value": 14.708152042710074,
            "unit": "MB/s",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 28.41365025065969,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "33842784+JakeDern@users.noreply.github.com",
            "name": "Jake Dern",
            "username": "JakeDern"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "3d3e119651dbd156e868da9529609f4e95c28038",
          "message": "feat(comparison_dashboard): Specify metrics metadata via manifest (#3011)\n\n# Change Summary\n\nThis PR makes the dashboard a little more generic by also pulling out\ntest metric related parameters to the manifest.\n\nWaiting on #3008 for a rebase.\n\n## What issue does this PR close?\n\n* Closes #3007\n\n## How are these changes tested?\n\n<img width=\"2320\" height=\"864\" alt=\"image\"\nsrc=\"https://github.com/user-attachments/assets/d276c080-0db4-4676-ab10-791becf423f0\"\n/>\n\n## Are there any user-facing changes?\n\nNo.\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2026-05-18T20:08:57Z",
          "tree_id": "97f412527963656607706ec41249ae4878b8e0d7",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/3d3e119651dbd156e868da9529609f4e95c28038"
        },
        "date": 1779144214455,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.6145119071006775,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.18556485565473,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.07935683364255,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 31.597265625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 32.85546875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 541537.8904932886,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 544865.705466776,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003336,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_mb_per_sec",
            "value": 14.778276755109035,
            "unit": "MB/s",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_mb_per_sec",
            "value": 14.731573779571058,
            "unit": "MB/s",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 28.440304044259786,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "33842784+JakeDern@users.noreply.github.com",
            "name": "Jake Dern",
            "username": "JakeDern"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "f562f907e0015b0bfda363415acc1c58aa407f44",
          "message": "feat: Replace static musl build on alpine with glibc based build on gcr distroless (#2919)\n\n# Change Summary\n\nThis PR changes our standard df_engine image from being a static build\nbased on musl + alpine to being based on glibc on gcr distroless.\n\nPart of this change requires updates to the mount paths for orchestrator\nconfig files as the home directory is now `nonroot` instead of\n`dataflow`.\n\n## What issue does this PR close?\n\n* Closes #2918\n\n## How are these changes tested?\n\nTo test this, I did local runs of all the nightly/continuous/comparison\ndashboard suites using the new image + mount path changes.\n\nI did my best to test cross compiling with the new targets for ARM as\nwell:\n\n<img width=\"1900\" height=\"91\" alt=\"image\"\nsrc=\"https://github.com/user-attachments/assets/03ce32f7-802c-42ea-b7bb-ada178cd71c7\"\n/>\n\n## Are there any user-facing changes?\n\nYes, the runtime image has been changed for the repo dockerfile and that\ncomes along with some new expectations for mount paths.\n\n---------\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2026-05-18T20:24:57Z",
          "tree_id": "81033348e230301298791687c738a1bd33b74846",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/f562f907e0015b0bfda363415acc1c58aa407f44"
        },
        "date": 1779145239307,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.3821854591369629,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 95.23893226967044,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 97.993750966744,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 32.56106770833333,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 33.4140625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 546998.8746126654,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 544908.3243548055,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003341,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_mb_per_sec",
            "value": 14.85419371950478,
            "unit": "MB/s",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_mb_per_sec",
            "value": 14.78444565569712,
            "unit": "MB/s",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 28.58416790029,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "33842784+JakeDern@users.noreply.github.com",
            "name": "Jake Dern",
            "username": "JakeDern"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "a1e7f18c5ff7263b5db00f6d0927bb3d9dcf1531",
          "message": "task(comparison_dashboard): Don't reset the focused metric when filters are changed (#3020)\n\n# Change Summary\n\nFix the dashboard so that the focused metric is not updated when filters\nare changed.\n\nAlso made a couple tweaks to the comparison definition to be able to set\nthe default metric.\n\n## What issue does this PR close?\n\n* Closes #3017\n\n## How are these changes tested?\n\nBy hand\n\n## Are there any user-facing changes?\n\nYes, slightly new behavior.\n\n---------\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2026-05-18T21:52:31Z",
          "tree_id": "5c21125bc4d24e353c11e0a45585d611caec45b3",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/a1e7f18c5ff7263b5db00f6d0927bb3d9dcf1531"
        },
        "date": 1779146274307,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.03897419944405556,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 95.44285738555143,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.88168264769564,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 33.67200520833333,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 34.2265625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 547347.79665123,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 547134.472231302,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002507,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_mb_per_sec",
            "value": 14.935144071392775,
            "unit": "MB/s",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_mb_per_sec",
            "value": 14.867547686957444,
            "unit": "MB/s",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 28.623006636628432,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
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
          "id": "95da33f167e5e67a3ec6a8303066e50347269db3",
          "message": "ci: add weaver live-check job for host_metrics_receiver (#2986)\n\nAdds a new `host-metrics-weaver-live-check` CI job that runs the\n`host_metrics_receiver` end-to-end against `weaver registry live-check`\n(OTLP listener mode) and fails on any `violation` finding.\n\nAdditive only: the existing `host-metrics-semconv` job is unchanged and\nthe new job is intentionally **not** in\n`rust-required-status-check.needs` for now. Goal is to let both\napproaches run side-by-side for a while before deciding which to keep\nand cleaning up.",
          "timestamp": "2026-05-18T22:13:50Z",
          "tree_id": "6136aeed54d0ddf549384f23c0d7f55af3981ee3",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/95da33f167e5e67a3ec6a8303066e50347269db3"
        },
        "date": 1779147615910,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.012756109237671,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 95.25484064497883,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.84691179857221,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 35.143489583333334,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 36.39453125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 551838.7564803174,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 546249.9758773857,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.005934,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_mb_per_sec",
            "value": 14.897871299957162,
            "unit": "MB/s",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_mb_per_sec",
            "value": 14.830331770968458,
            "unit": "MB/s",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 28.597805008837895,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "29139614+renovate[bot]@users.noreply.github.com",
            "name": "renovate[bot]",
            "username": "renovate[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "1da4d0859baa412af04b9374dd5b91f76bc62a47",
          "message": "chore(deps): refresh pip-compile outputs (#2924)\n\nThis PR contains the following updates:\n\n| Update | Change |\n|---|---|\n| lockFileMaintenance | All locks refreshed |\n\n🔧 This Pull Request updates lock files to use the latest dependency\nversions.\n\n---\n\n### Configuration\n\n📅 **Schedule**: (UTC)\n\n- Branch creation\n  - \"before 4am on monday\"\n- Automerge\n  - At any time (no schedule defined)\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n👻 **Immortal**: This PR will be recreated if closed unmerged. Get\n[config\nhelp](https://redirect.github.com/renovatebot/renovate/discussions) if\nthat's undesired.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My4xNTkuMiIsInVwZGF0ZWRJblZlciI6IjQzLjE3My42IiwidGFyZ2V0QnJhbmNoIjoibWFpbiIsImxhYmVscyI6WyJkZXBlbmRlbmNpZXMiXX0=-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2026-05-18T23:04:53Z",
          "tree_id": "a8fdc6f7aaa7c1d926fda42d5beb4973ee488c2c",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/1da4d0859baa412af04b9374dd5b91f76bc62a47"
        },
        "date": 1779148847114,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.7138423323631287,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 95.4349214931185,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.31946773943989,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 35.269921875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 36.71484375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 549868.8370710281,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 545943.6405307942,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002091,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_mb_per_sec",
            "value": 14.899879173204033,
            "unit": "MB/s",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_mb_per_sec",
            "value": 14.835428988885019,
            "unit": "MB/s",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 28.617708027025426,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "wbutler@microsoft.com",
            "name": "Will Butler",
            "username": "wbutler"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "f89428238800fc80fe7929104a2a300d26963202",
          "message": "Better config-time validation for overlapping flow metrics configuration (#2983)\n\nCloses #2784.\n\nThis change implements a BFS walk of the processor pipeline at\nconstruction time to validate that flow metrics do not collide.\n\n- In `runtime_pipeline.rs`, introduce a helper func to capture a flat\nedge list vector of pipeline connections and pass the result as a new\nparam on `build_flow_metric_state`. Saves us from having to pass in some\npart of `_config` into that function and keeps data types better\nseparated.\n- In `flow_metrics.rs`:\n- In `build_flow_metric_state`, add a new data structure that captures\nstart\\end pairs for registered flow metrics.\n  - at the end of the main loop, iff the count of flow metrics > 1\n- convert the list of edges into a one-to-many adjacency data structure\n    - for each flow metric\n      - do a BFS to find all the nodes between the start and the end\n- check for nodes in this set that also appear as a start node in the\ndata structure above.\n      - If found, throw an error, halting pipeline construction.\n\n## Tests\n\nWithin `flow_metrics.rs`, this change adds 10 unit tests, as follows:\n- 7 tests exercise the interleaving detection logic on synthetically\ncreated pipeline topologies.\n  - Three expected-pass tests include:\n    - Linear topology with disjoint ranges\n    - Branching topology with disjoint ranges\n    - Linear topology with a single flow metric\n  - Four expected-reject tests include:\n    - Linear topology with interleaving\n    - Linear topology with interleaving, reverse declaration order\n    - Diamond topology with interleaving\n    - Linear topology with fully nested ranges\n- 3 tests exercise a helper function directly.\n\n## Validation\n\nThe following commands pass cleanly:\n\n`cargo check -p otap-df-engine`\n`cargo test -p otap-df-engine` (contains new tests)\n`cargo clippy -p otap-df-engine --all-targets -- -D warnings`\n`cargo fmt --all -- --check`\n`cargo xtask quick-check`",
          "timestamp": "2026-05-18T23:21:18Z",
          "tree_id": "8bf8eb494dd43929ebb06d2eb1085b482496217a",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/f89428238800fc80fe7929104a2a300d26963202"
        },
        "date": 1779149657340,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.6462161540985107,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 95.41041125097983,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.9038761609907,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 34.58033854166667,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 35.36328125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 547953.6794840003,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 544412.7144110346,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00624,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_mb_per_sec",
            "value": 14.858830020400045,
            "unit": "MB/s",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_mb_per_sec",
            "value": 14.793328591388102,
            "unit": "MB/s",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 28.619119530165033,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
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
          "id": "2d53d89e89d477a39c38d39d863e942167f122ea",
          "message": "docs: document saturation test workload characteristics (#3021)\n\nDocument that saturation tests use static 1KB log bodies with realistic\nentropy (512 unique bodies), distinguishing them from other tests that\nuse semantic_conventions (~300 byte logs). Also removes the stale TODO\nand adds scaling efficiency formula explanation with link to the\nscaling-efficiency benchmark page.",
          "timestamp": "2026-05-18T23:28:48Z",
          "tree_id": "bd7ed8577cdc4de42a408750f488aa1338783ddd",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/2d53d89e89d477a39c38d39d863e942167f122ea"
        },
        "date": 1779150470672,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.6380826234817505,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 95.33720825776236,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.35748458692971,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 35.110416666666666,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 36.015625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 548279.6714031105,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 544781.1940040711,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003246,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_mb_per_sec",
            "value": 14.88042198545571,
            "unit": "MB/s",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_mb_per_sec",
            "value": 14.816011943513695,
            "unit": "MB/s",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 28.6413215719495,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "29139614+renovate[bot]@users.noreply.github.com",
            "name": "renovate[bot]",
            "username": "renovate[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "4444df4851a9f08b691f60780065a472b7a0ac6a",
          "message": "chore(deps): update rust crate sha1 to 0.11 (#2998)\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n|---|---|---|---|\n| [sha1](https://redirect.github.com/RustCrypto/hashes) |\nworkspace.dependencies | minor | `0.10` → `0.11` |\n\n---\n\n### Release Notes\n\n<details>\n<summary>RustCrypto/hashes (sha1)</summary>\n\n###\n[`v0.11.0`](https://redirect.github.com/RustCrypto/hashes/compare/sha2-v0.10.9...sha2-v0.11.0)\n\n[Compare\nSource](https://redirect.github.com/RustCrypto/hashes/compare/sha1-v0.10.6...sha1-v0.11.0)\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: (UTC)\n\n- Branch creation\n  - \"before 8am on Monday\"\n- Automerge\n  - At any time (no schedule defined)\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My4xNzkuMyIsInVwZGF0ZWRJblZlciI6IjQzLjE4Mi4yIiwidGFyZ2V0QnJhbmNoIjoibWFpbiIsImxhYmVscyI6WyJkZXBlbmRlbmNpZXMiXX0=-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-05-19T02:19:31Z",
          "tree_id": "9021873592f804a4587240ddffa446045916cf7a",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/4444df4851a9f08b691f60780065a472b7a0ac6a"
        },
        "date": 1779161783566,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.6627163290977478,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 95.24691595368554,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.70236562475762,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 35.504296875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 36.421875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 547185.1392690248,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 543558.8538458324,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.006308,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_mb_per_sec",
            "value": 14.838239033879244,
            "unit": "MB/s",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_mb_per_sec",
            "value": 14.771427061893364,
            "unit": "MB/s",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 28.624354516727106,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
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
          "id": "e62aa2647675f736590bd31b9123e958c01d6291",
          "message": "Skip continuous benchmark when only docs/non-rust files change (#3028)\n\nAdds `paths` filter to the continuous pipeline performance test workflow\nso it only runs when relevant code is modified:\n\n- `rust/**`\n- `tools/pipeline_perf_test/**`\n- `tools/comparison_dashboard/**`\n- The workflow file itself\n\nThis avoids wasting expensive bare-metal runner time on commits that\nonly touch markdown, Go code, or other unrelated files.\n`workflow_dispatch` still allows manual runs anytime.\n\n---------\n\nCo-authored-by: Copilot Autofix powered by AI <175728472+Copilot@users.noreply.github.com>\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2026-05-19T03:30:02Z",
          "tree_id": "23fcd7b54e8f873ab923f7c82e2842d0ae2b96ad",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/e62aa2647675f736590bd31b9123e958c01d6291"
        },
        "date": 1779163952437,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.5236011743545532,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 95.39092425255225,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.05229383444635,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 35.428776041666666,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 36.19140625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 545941.7479986892,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 543083.1908624035,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002299,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_mb_per_sec",
            "value": 14.83972347413782,
            "unit": "MB/s",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_mb_per_sec",
            "value": 14.774598273489726,
            "unit": "MB/s",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 28.652291478415496,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
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
          "distinct": false,
          "id": "2688ea1753a3c8717044a4fb5a8cfb49e0a6b333",
          "message": "Add single-core saturation benchmark (semantic_conventions payload) (#3023)\n\n## Summary\n\nAdds single-core saturation tests that measure maximum throughput per\ncore using the same payload as all other benchmarks\n(`semantic_conventions`, ~300 byte logs). Tests both OTLP and OTAP\nprotocols as separate test configs.\n\n### Results (single core, fully saturated)\n\n| Protocol | Max Throughput | CPU (avg) | RAM (avg) |\n|----------|---------------|-----------|-----------|\n| **OTLP-ATTR-OTLP** | **360K logs/sec** | 100% | 23 MiB |\n| **OTAP-ATTR-OTAP** | **2.58M logs/sec** | 100% | 48 MiB |\n\nOTAP is **7.2x faster** than OTLP on a single core using the Arrow\nprotocol.\n\n### Test configs\n\n**OTLP (`saturation-max-throughput.yaml`):**\n- SUT: 1 core (core 0, NUMA0)\n- Loadgen: 4 cores (cores 32-35, NUMA1), unleashed\n- Backend: 2 cores (cores 36-37, NUMA1)\n- `num_connections: 4`\n\n**OTAP (`saturation-max-throughput-otap.yaml`):**\n- SUT: 1 core (core 0, NUMA0)\n- Loadgen: 8 cores (cores 32-39, NUMA1), unleashed\n- Backend: 4 cores (cores 40-43, NUMA1)\n- OTAP needs more loadgen/backend cores because the Arrow protocol is\n~7x more efficient\n\nBoth use NUMA-aware pinning: SUT on NUMA node 0, loadgen+backend on NUMA\nnode 1 to avoid L3 cache contention.\n\n### Changes\n- New: `saturation-max-throughput.yaml` (OTLP single-core saturation)\n- New: `saturation-max-throughput-otap.yaml` (OTAP single-core\nsaturation)\n- Modified: `pipeline-perf-test-nightly.yml` -- added both tests to\nnightly runs\n- Modified: `docs/benchmarks.md` -- added section 6a (Max Throughput)\nand renamed existing section to 6b (Scaling Efficiency)\n- `pipeline-perf-on-label.yaml` -- no changes (reverted validation-only\nmodifications)\n\nCloses: #3022",
          "timestamp": "2026-05-19T04:47:25Z",
          "tree_id": "a5c262435ca8dbc5d74f35ce803963baefdafb86",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/2688ea1753a3c8717044a4fb5a8cfb49e0a6b333"
        },
        "date": 1779170198619,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.4579867124557495,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.12439948266343,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.44172889233151,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 34.72044270833333,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 35.41796875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 547769.2994129409,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 545260.5887230415,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002136,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_mb_per_sec",
            "value": 14.923133584861556,
            "unit": "MB/s",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_mb_per_sec",
            "value": 14.890193104624654,
            "unit": "MB/s",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 28.69827756766045,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
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
          "distinct": false,
          "id": "5bcb1d163c2f312697327ac867f83be7820fc6f2",
          "message": "fix(perf): update df_engine mount path to /home/nonroot in syslog-tcp-otelcol nightly suite (#3032)\n\nThe nightly Pipeline Performance Tests run\n[26069747923](https://github.com/open-telemetry/otel-arrow/actions/runs/26069747923/job/76648381259)\nfailed in the \"Run syslog TCP performance test otelcol log suite\" step\nbecause the backend (`df_engine`) container never became ready\n(readiness check timed out after 10 attempts).\n\nRoot cause: PR #2919 switched the `df_engine` image from musl/alpine to\nglibc/distroless, changing the in-container home directory from\n`/home/dataflow` to `/home/nonroot`, and updated the volume mount path\nin all then-existing nightly docker yamls. The\n`syslog-tcp-otelcol-docker.yaml` suite (added concurrently in #2962) was\nmissed, so it was still mounting the backend config at\n`/home/dataflow/config.yaml` — a path that doesn't exist in the new\nimage — causing the backend container to fail to start.\n\nThis one-line change brings it in line with the other nightly suites.",
          "timestamp": "2026-05-19T05:56:58Z",
          "tree_id": "06f513c3615b38928d130b5e68c1608a83bf089e",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/5bcb1d163c2f312697327ac867f83be7820fc6f2"
        },
        "date": 1779174672757,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.2066365480422974,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 95.40529919247956,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 97.925476832935,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 35.367838541666664,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 35.9140625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 537362.3691206599,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 543846.379607787,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.012241,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_mb_per_sec",
            "value": 14.836118503163158,
            "unit": "MB/s",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_mb_per_sec",
            "value": 14.77132474669836,
            "unit": "MB/s",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 28.605132586875207,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
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
          "distinct": false,
          "id": "475bf0574e3c0ea317103d55538cb54acef3e698",
          "message": "NUMA-aware passthrough test with OTAP variant (#3027)\n\n## Summary\n\nFix passthrough benchmark test with NUMA-aware CPU pinning and add\nOTAP-OTAP variant.\n\n## Changes\n\n1. **NUMA-aware passthrough (OTLP)**: Pin SUT to core 0, loadgen to\ncores 32-35, backend to cores 36-37 (cross-NUMA). Increase batch size\n512→1000 and add num_connections: 4.\n2. **New OTAP passthrough test**: Add `passthrough-otap.yaml` with OTAP\nreceiver/exporter engine config. Uses 8 loadgen cores (32-39) and 4\nbackend cores (40-43) to saturate single-core SUT.\n3. **Add OTAP to continuous workflow**: Both passthrough variants run in\nthe continuous CI pipeline.\n\n## Results (CI validated, run 26076764277)\n\n| Test | Throughput | CPU | Notes |\n|------|-----------|-----|-------|\n| Passthrough OTLP (before, no NUMA) | 263K logs/sec | ~70% | Loadgen\nbottleneck |\n| Passthrough OTLP (after, NUMA-aware) | **607K logs/sec** | 100% |\nFully saturated |\n| Passthrough OTAP (NUMA-aware) | **2.64M logs/sec** | 100% | Fully\nsaturated |\n| With ATTR processor - OTLP (PR #3023) | 360K logs/sec | 100% |\nDeser/reser overhead |\n| With ATTR processor - OTAP (PR #3023) | 2.58M logs/sec | 100% |\nIn-place processing |\n\n### Key Observations\n\n- **NUMA fix alone gives 2.3x improvement** for OTLP passthrough (263K →\n607K)\n- **OTAP is 4.3x faster than OTLP** in passthrough mode\n- **ATTR processor overhead**: ~41% for OTLP (607K → 360K, due to\ndeserialization/reserialization), <2% for OTAP (2.64M → 2.58M, in-place\nprocessing)\n- Both tests now properly saturate a single core\n\n## Related\n\n- Closes part of performance benchmarking improvements\n- PR #3023 adds saturation-max-throughput tests (with ATTR processor)",
          "timestamp": "2026-05-19T06:14:49Z",
          "tree_id": "827b67f453f46e75a78b67545b29e8ef25f5759c",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/475bf0574e3c0ea317103d55538cb54acef3e698"
        },
        "date": 1779175605559,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.5509935617446899,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.25904713523373,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.54550244053615,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 45.493229166666666,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 47.06640625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 2658591.5787391067,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 2643942.910791312,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.005456,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_mb_per_sec",
            "value": 19.314273373375027,
            "unit": "MB/s",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_mb_per_sec",
            "value": 19.29578425236385,
            "unit": "MB/s",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 7.659954923421051,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 3.0598857402801514,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 99.71303631389765,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.3478940027894,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 26.584895833333334,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 27.69921875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 634992.9719345805,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 615562.91353759,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.010113,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_mb_per_sec",
            "value": 15.606177383931135,
            "unit": "MB/s",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_mb_per_sec",
            "value": 15.607536130558897,
            "unit": "MB/s",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 26.584225099736575,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "AaronRM@users.noreply.github.com",
            "name": "Aaron Marten",
            "username": "AaronRM"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "c5e745b948f26b9f6a9a347c29fea6ef3c9430c8",
          "message": "fix(tests): improve retry pipeline tests with deterministic auto-flip for NACKs (#3030)\n\n# Change Summary\n\n- Adds a deterministic 'auto flip to NACK' mode to flaky-exporter\n- Use the new 'auto flip to NACK' mode in\n`durable_buffer_processor_tests` and `core_node_liveness_tests` to\nimprove test reliability.\n\n## What issue does this PR close?\n\n* Addresses flaky test failure for\n`core_node_liveness_tests::test_retry_pipeline_eventually_recovers_after_transient_nacks`\nin #2720.\n\n## How are these changes tested?\n\n* Validated that tests pass locally on multiple re-runs.\n\n## Are there any user-facing changes?\n\nNo. These are test-only changes.\n\n---------\n\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-05-19T14:34:15Z",
          "tree_id": "c9c2cca5392448e207e41be11f57e36b7f0f3e46",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/c5e745b948f26b9f6a9a347c29fea6ef3c9430c8"
        },
        "date": 1779205810365,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.12353033572435379,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.27942383819067,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.60105964368707,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 49.83971354166667,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 60.2890625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 2657672.913765926,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 2660955.945990487,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.005503,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_mb_per_sec",
            "value": 19.484650881982507,
            "unit": "MB/s",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_mb_per_sec",
            "value": 19.454123359233805,
            "unit": "MB/s",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 7.6781193292625565,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.6340847611427307,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 99.69466879377691,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.30880372381692,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 26.895182291666668,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 27.57421875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 620281.5070432425,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 624214.6174347685,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003401,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_mb_per_sec",
            "value": 15.536052828416322,
            "unit": "MB/s",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_mb_per_sec",
            "value": 15.543590617881845,
            "unit": "MB/s",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 26.097966429489908,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
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
          "distinct": false,
          "id": "3b7e851df5e8ebbd346cf0d758c3a5f7f6f50188",
          "message": "Add engine self-reported RSS to idle state benchmark report (#3031)\n\nAdds the engine's self-reported `memory_rss` metric (process Resident\nSet Size) to the idle state benchmark report, alongside the existing\nDocker container cgroup memory (`container.memory.usage`).\n\nThe engine exposes `memory_rss` via its Prometheus endpoint — this is\nthe actual process RSS matching what `ps rss` / `htop` would show,\nwithout kernel caches that inflate cgroup memory. Having both gives a\nclearer picture of actual memory footprint.\n\nNew output metrics: `idle_rss_mib_avg` and `idle_rss_mib_max`.\n\nNote: If this works well in CI, we can expand to other test suites\n(integration, comparison dashboard, saturation) in a follow-up. For load\ntests the cgroup vs RSS gap is smaller, so idle is the highest-value\nstarting point.",
          "timestamp": "2026-05-19T14:45:54Z",
          "tree_id": "9bc79eb567b6936f0f419f6e2825c2aa0da3dc8a",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/3b7e851df5e8ebbd346cf0d758c3a5f7f6f50188"
        },
        "date": 1779208416932,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 2.6100785732269287,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.27053134134745,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.64989449364651,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 45.849609375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 47.84765625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 2670991.605216979,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 2601276.6253751866,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.03014,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_mb_per_sec",
            "value": 19.34026846473319,
            "unit": "MB/s",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_mb_per_sec",
            "value": 19.311216782107994,
            "unit": "MB/s",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 7.796072569848693,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.30123692750930786,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 99.52200389372733,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.19985130111525,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 26.720833333333335,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 27.7578125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 625167.9184663174,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 623284.6817744136,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003079,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_mb_per_sec",
            "value": 15.511545295076669,
            "unit": "MB/s",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_mb_per_sec",
            "value": 15.5176462284762,
            "unit": "MB/s",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 26.095674408403227,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
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
          "id": "31f098bdc0a638db417fd95bed5bf60147545198",
          "message": "perf: keep network rate metrics in bytes/sec for dashboard formatting (#3035)\n\nFollow-up to #2982 per [review\nsuggestion](https://github.com/open-telemetry/otel-arrow/pull/2982#issuecomment-4464515872):\nkeep report data in bytes/sec and let the dashboard handle\nrounding/reformatting into MiB/s or GiB/s as needed.",
          "timestamp": "2026-05-19T15:01:04Z",
          "tree_id": "23006f68407c25c16aa76222bcc4f2c3485a99f0",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/31f098bdc0a638db417fd95bed5bf60147545198"
        },
        "date": 1779210022688,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.22650767862796783,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.23198243981997,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.5775195599969,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 46.98020833333333,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 52.41796875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 2648681.243048847,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 2642681.7763014496,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.005333,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 20281941.53902673,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 20246116.40720107,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 7.674757407761826,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.14143894612789154,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 99.65185745682741,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.30300411842413,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 26.983854166666667,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 28.19140625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 624496.7794551759,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 625380.061087091,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003512,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 16292693.01367426,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 16304821.262459457,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 26.05246637597121,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "29139614+renovate[bot]@users.noreply.github.com",
            "name": "renovate[bot]",
            "username": "renovate[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "39c8b738e5c0b91fb4c4b747d129b2847b4921f7",
          "message": "fix(deps): update module go.opentelemetry.io/collector/pdata to v1.58.0 - abandoned (#2999)\n\nThis PR contains the following updates:\n\n| Package | Change |\n[Age](https://docs.renovatebot.com/merge-confidence/) |\n[Confidence](https://docs.renovatebot.com/merge-confidence/) |\n|---|---|---|---|\n|\n[go.opentelemetry.io/collector/pdata](https://redirect.github.com/open-telemetry/opentelemetry-collector)\n| `v1.57.0` → `v1.58.0` |\n![age](https://developer.mend.io/api/mc/badges/age/go/go.opentelemetry.io%2fcollector%2fpdata/v1.58.0?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/go/go.opentelemetry.io%2fcollector%2fpdata/v1.57.0/v1.58.0?slim=true)\n|\n\n---\n\n### Release Notes\n\n<details>\n<summary>open-telemetry/opentelemetry-collector\n(go.opentelemetry.io/collector/pdata)</summary>\n\n###\n[`v1.58.0`](https://redirect.github.com/open-telemetry/opentelemetry-collector/blob/HEAD/CHANGELOG.md#v1580v01520)\n\n##### 💡 Enhancements 💡\n\n- `pkg/exporterhelper`: Add `otelcol_exporter_in_flight_requests` metric\nto track the number of export requests currently in-flight per exporter.\n([#&#8203;15009](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/15009))\nThis UpDownCounter increments in startOp and decrements in endOp,\nallowing operators to monitor\nconcurrent export activity and detect when an exporter is saturating its\nworker pool.\n\n##### 🧰 Bug fixes 🧰\n\n- `pkg/confighttp`: Close the original request body after reading\nblock-format `Content-Encoding: snappy` requests.\n([#&#8203;15262](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/15262))\n\n- `pkg/confighttp`: Recover from panics in decompression libraries,\nreturn HTTP 400 instead of 500.\n([#&#8203;13228](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/13228))\n\n- `pkg/confighttp`: Enforce `max_request_body_size` on\n`Content-Encoding: snappy` requests before the decoded buffer is\nallocated.\n([#&#8203;15252](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/15252))\n\n- `pkg/otelcol`: Stop emitting verbose gRPC transport messages at WARN\nduring normal client disconnect.\n([#&#8203;5169](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/5169))\ngrpc-go gates chatty per-RPC notices (e.g. \"HandleStreams failed to read\nframe:\nconnection reset by peer\") behind `LoggerV2.V(2)`. zapgrpc.Logger.V\nconflates\ngrpclog verbosity with zap severity, so V(2) returns true whenever WARN\nis\nenabled and these messages emit at WARN. Wrap the installed\ngrpclog.LoggerV2\nwith a corrected V() that compares against a fixed verbosity threshold,\nmatching grpclog's intended semantics. See\n[uber-go/zap#1544](https://redirect.github.com/uber-go/zap/issues/1544).\n\n- `pkg/pdata`: `pcommon.Value.AsString` no longer HTML-escapes `<`, `>`,\nand `&` inside `ValueTypeMap` and `ValueTypeSlice` values, matching the\nbehavior already used for `ValueTypeStr`.\n([#&#8203;14662](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/14662))\n\n- `pkg/service`: Fix Prometheus config defaults mismatch when host is\nexplicitly set in telemetry configuration.\n([#&#8203;13867](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/13867))\nWhen users explicitly configured the telemetry metrics section (e.g. to\nchange the host),\nthe Prometheus exporter boolean fields (WithoutScopeInfo, WithoutUnits,\nWithoutTypeSuffix)\ndefaulted to nil/false instead of true, causing metric name format\nchanges compared to the\nimplicit default configuration. This fix applies the correct defaults\nduring config unmarshaling.\n\n- `pkg/service`: Return noop tracer provider when no trace processors\nare defined\n([#&#8203;15135](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/15135))\n\n<!-- previous-version -->\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: (UTC)\n\n- Branch creation\n  - \"before 8am on Monday\"\n- Automerge\n  - At any time (no schedule defined)\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My4xNzkuMyIsInVwZGF0ZWRJblZlciI6IjQzLjE3OS4zIiwidGFyZ2V0QnJhbmNoIjoibWFpbiIsImxhYmVscyI6WyJkZXBlbmRlbmNpZXMiXX0=-->\n\n---------\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: otelbot <197425009+otelbot@users.noreply.github.com>",
          "timestamp": "2026-05-19T16:55:41Z",
          "tree_id": "8089f53dcc0c488aaff9dcb71f691faf60a8f015",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/39c8b738e5c0b91fb4c4b747d129b2847b4921f7"
        },
        "date": 1779217432718,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.743205726146698,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.17529394938158,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.46018682930595,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 26.662890625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 27.56640625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 600970.652599798,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 596504.2040447025,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00293,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 15847584.353055201,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 15833648.133347547,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 26.56743111883847,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.6629270911216736,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.2236190567337,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.56900400863398,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 46.584375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 51.19140625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 2613727.0317987353,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 2596399.9275593758,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.02157,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 20253883.34690982,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 20231022.07311346,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 7.800756398090234,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "42589365+simathih@users.noreply.github.com",
            "name": "Siddhartha Mathiharan",
            "username": "simathih"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "c4df8317b2606d32e446ae1ff6766da264c66cb4",
          "message": "Rust decoder shouldn't assume root payload is at position 0 (#2468)\n\n# Change Summary\n\nRust decoder shouldn't assume root payload is at position 0.\nWe know what root payload we're expecting so we can update the code to\nfill in the appropriate Logs/Metrics/Traces construct and then check for\nroot payload presence at the end.\n\n## What issue does this PR close?\n\nhttps://github.com/open-telemetry/otel-arrow/issues/2363\n\n* Closes #NNN\n\n## How are these changes tested?\n\n## Are there any user-facing changes?\n\n <!-- If yes, provide further info below -->\n\n---------\n\nCo-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>\nCo-authored-by: Cijo Thomas <cijo.thomas@gmail.com>\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-05-19T18:36:47Z",
          "tree_id": "04e3b95791127926c289ce595a60d8cb8f7e6f0a",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/c4df8317b2606d32e446ae1ff6766da264c66cb4"
        },
        "date": 1779220520436,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.3197549283504486,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 99.40989205404259,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.35505422993492,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 26.936328125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 27.4140625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 620236.5851922849,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 622219.8223386166,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00291,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 16226310.275753338,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 16247044.251221796,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 26.0780992395367,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.3953341841697693,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.21319706962653,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.57356783140932,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 45.53541666666667,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 46.3125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 2643188.3104013694,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 2653637.737076159,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003292,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 20365632.33415734,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 20331409.02730317,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 7.674609103425201,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "33842784+JakeDern@users.noreply.github.com",
            "name": "Jake Dern",
            "username": "JakeDern"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "776d98488ce3613c611238347e28511f9f842f97",
          "message": "feat(otlp_http_exporter): Add request compression options (#3040)\n\n# Change Summary\n\nThis PR implements gzip, zstd, and deflate compression options to match\nthose available in other exporters such as otlp_grpc and otap.\n\n## What issue does this PR close?\n\n* Closes #3012\n\n## How are these changes tested?\n\nUnit tests and benchmarks which show we're inline with expectations:\n\nThe perf results show that we're in line with expectations:\n\n| Rate | Compression | TX MB/s (gRPC) | TX MB/s (HTTP) | CPU avg %\n(gRPC) | CPU avg % (HTTP) | RAM max MiB (gRPC) | RAM max MiB (HTTP) |\n\n|------|-------------|---------------:|---------------:|-----------------:|-----------------:|-------------------:|-------------------:|\n| 100k | none | 36.0 | 35.9 | 3.9 | 1.6 | 55.2 | 48.9 |\n| 100k | gzip | 2.6 | 2.6 | 16.6 | 12.9 | 52.0 | 54.6 |\n| 100k | zstd | 1.8 | 1.8 | 5.9 | 4.0 | 54.7 | 53.3 |\n| 400k | none | 143.8 | 143.0 | 14.0 | 5.8 | 54.3 | 52.9 |\n| 400k | gzip | 10.4 | 10.4 | 62.3 | 50.9 | 50.7 | 57.0 |\n| 400k | zstd | 7.0 | 7.1 | 23.2 | 15.5 | 55.8 | 52.5 |\n| 800k | none | 288.6 | 286.9 | 27.5 | 11.4 | 53.5 | 54.3 |\n| 800k | gzip | 16.1 | 20.6 | 100.0 | 99.8 | 106.4 | 69.8 |\n| 800k | zstd | 13.9 | 14.0 | 46.6 | 32.2 | 57.7 | 55.0 |\n\n## Are there any user-facing changes?\n\nYes new config.\n\n---------\n\nCo-authored-by: Copilot Autofix powered by AI <175728472+Copilot@users.noreply.github.com>",
          "timestamp": "2026-05-19T20:41:18Z",
          "tree_id": "b8d860731c9a61e7b4d7ad6612cf7985681670c3",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/776d98488ce3613c611238347e28511f9f842f97"
        },
        "date": 1779230959277,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.7151917815208435,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.20705117483122,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.49891089108911,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 27.216666666666665,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 27.9453125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 617519.1755011562,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 613102.729288763,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002995,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 15980885.76771173,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 15969465.187996816,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 26.06559228051479,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 2.2394304275512695,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.23291140738048,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.6432932038835,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 45.20234375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 47.5859375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 2668688.245649045,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 2608924.828120178,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003262,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 20373446.780295484,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 20352144.10252168,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 7.809135227163777,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "mblanchard@macrosssoftware.com",
            "name": "Mikel Blanchard",
            "username": "CodeBlanch"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "cdd4bb24c2e7ddc26afc4461bcfb90a475f8ae8c",
          "message": "[pdata] [query-engine] Expose LogsBodyBuilder::finish as pub (#3044)\n\n# Changes\n\n* Expose `LogsBodyBuilder::finish` as `pub`\n\n# Details\n\nI would like to use `LogsBodyBuilder` in OTAP columnar query engine but\n`finish` is currently private. It seems the other `finish` methods in\nhere are all `pub` so I'm hoping this is OK to change\\expose.\n\n/cc @albertlockett\n\n---------\n\nCo-authored-by: albertlockett <a.lockett@f5.com>\nCo-authored-by: Copilot Autofix powered by AI <175728472+Copilot@users.noreply.github.com>",
          "timestamp": "2026-05-19T21:26:21Z",
          "tree_id": "4d8ccf3143a51691155825e1707cf6bcaf999738",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/cdd4bb24c2e7ddc26afc4461bcfb90a475f8ae8c"
        },
        "date": 1779232118408,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.2594919502735138,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.20128539600799,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.48213052566386,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 26.976432291666665,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 27.91796875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 610112.7021981572,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 611695.8954980432,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.005307,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 15984454.202720933,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 15982992.987882273,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 26.131373972529897,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.3447684049606323,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.2325939773099,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.55423820992931,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 44.88098958333333,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 47.4296875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 2671709.0714209117,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 2635780.771043656,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.008405,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 20247581.51049406,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 20229724.42117962,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 7.68181547302088,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "geukhanuslu@gmail.com",
            "name": "Gokhan Uslu",
            "username": "gouslu"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "60f251825b8a2022b3c3373761eb6b55f9a30da0",
          "message": "feat(engine): wire extensions and capabilities into runtime pipeline (#2860)\n\n# Change Summary\n \n Part 4 of the Extension System (P1) series. Wires the previously\n landed Capability Registry & Resolver (#2732) into the runtime\n pipeline so extensions are actually instantiated, started, and\n shut down by the engine, and so consumer nodes can resolve their\n capability bindings at build time.\n \n Highlights:\n \n - **Runtime wiring** in `runtime_pipeline.rs`: extension lifecycle\n   is invoked before any data-path node is spawned, and `Shutdown`\n   is delivered to extensions only after the data path drains\n   (\"started first, shut down last\"). Active and passive extensions\n   are handled separately; failures abort startup cleanly.\n - **Local capability ownership aligned with shared** via a\n   Box-clone factory pattern, removing the prior asymmetry between\n   the two trait variants.\n - **Two reference test capabilities** under\n   `crates/engine/src/testing/capability/`: `NoOpStateless` and\n   `NoOpStateful`. They exercise every codegen path of the\n   `#[capability]` proc macro (`&self` × {sync, async}, `&mut self`\n   × {sync, async}, borrowed/owned returns, etc.). Test-only; they\n   intentionally live under `testing/` rather than the public\n   `capability/` surface.\n - **Comprehensive end-to-end test suite** at\n   `crates/engine/tests/extension_e2e.rs` (26 tests) covering:\n   passive/active/background extensions, lifecycle ordering and\n   shutdown ordering, fail-fast on extension errors, dual-variant\n   pruning, one-shot capability enforcement (all accessor\n   combinations), shared mutable state across consumers via\n   `Arc`/`Rc` for both local and shared trait variants, async\n   `&mut self` invocation through boxed handles, and active\n   extensions mutating shared state observed by capability\n   consumers.\n - **Architecture doc** updated with a precise statement of the\n   start-first/shut-down-last invariant (it orders lifecycle\n   *calls*, not init completion) and a noted future consideration\n   to add an opt-in readiness probe if/when an extension needs an\n   init-complete guarantee.\n - **URN unification**: extension URNs now use the canonical\n   4-segment form `urn:<namespace>:extension:<id>` (mirroring the\n   receiver/processor/exporter convention), with a short form\n   `extension:<id>`. The shared parser core lives in a new\n   private `crates/config/src/urn.rs`; `node_urn.rs` and\n   `extension_urn.rs` delegate to it with disjoint accepted-kind\n   sets so the two URN types cannot be confused. As a consequence,\n   `NodeKind::Extension` and the now-unreachable\n   `Error::ExtensionInNodesSection` are removed. Misplacement\n   errors include actionable hints (e.g. *\"declare under\n   `extensions:` instead of `nodes:`\"*).\n - All in-tree node factories (receivers, processors, exporters\n   in `core-nodes` and `contrib-nodes`) updated to accept the new\n   `&Capabilities` parameter; existing factories that don't depend\n   on any capability simply ignore it.\n \n ## What issue does this PR close?\n \n ## How are these changes tested?\n \n - New `extension_e2e.rs` integration test (26 tests) exercises the\n   wiring end-to-end against synthetic receivers/processors/\n   exporters/extensions.\n - New unit tests in `urn.rs` cover the shared parser core and the\n   misplacement-error hints; existing `extension_urn` and\n   `node_urn` tests updated to assert the canonical 4-segment form.\n - Pipeline-level regression tests cover rejecting extension URNs\n   in the `nodes:` section and node URNs in the `extensions:`\n   section.\n - `cargo xtask check` (structure check + `fmt` + `clippy --workspace\n   --all-targets -- -D warnings` + `cargo test --workspace`) passes\n   cleanly. No new clippy warnings.\n \n ## Are there any user-facing changes?\n \n Yes:\n \n - **Extension URN format**: extension URNs now use\n   `urn:<namespace>:extension:<id>` (4-segment) instead of the\n   pre-existing 3-segment `urn:<namespace>:<id>`. Short form\n   `extension:<id>` (expands to `urn:otel:extension:<id>`) is\n   available as a developer convenience. Existing 3-segment\n   extension URNs in pipeline configs must be updated. The\n   previously-bundled `configs/fake-with-extension.yaml` was an\n   orphan (its URN had no registered `ExtensionFactory` anywhere\n   in the binary, and it had no test/script/doc consumers) and\n   was removed in `482feb22c`; the canonical 4-segment shape is\n   covered by the `test_extension_with_config_and_capabilities`\n   unit test in `crates/config/src/pipeline.rs`. A runnable demo\n   config can land in a follow-up alongside a real factory.\n - **New extension authoring surface**: `Extension` trait,\n   `ExtensionWrapper::builder` typestate, the\n   `extension_capabilities!` macro, and the test capabilities\n   `NoOpStateless` / `NoOpStateful` (under `testing/capability/`)\n   are now reachable for external extension authors. The\n   architecture doc captures the lifecycle contract.\n - **Node factory signature** now includes `&Capabilities` as a\n   parameter; existing custom factories will need to accept (and\n   may ignore) this new argument\n\n---------\n\nCo-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>",
          "timestamp": "2026-05-19T23:25:05Z",
          "tree_id": "23ebb02ddf5c0b4513126adc8835358482d80da8",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/60f251825b8a2022b3c3373761eb6b55f9a30da0"
        },
        "date": 1779237424792,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 2.5982553958892822,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.25067964151889,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.5361143298571,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 47.08177083333333,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 48.2890625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 2665018.543139528,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 2595774.556979726,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.005211,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 20234541.3750001,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 20191531.32127955,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 7.795184416378475,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.059932440519332886,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.2000116152275,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.4729342248484,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 26.768880208333332,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 27.4765625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 611770.1558125656,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 612136.8045928826,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002927,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 16035003.761316653,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 16026477.756137185,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 26.19513095929781,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
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
          "id": "fabbe70cbf95113006d75b3e89725cd930d747c3",
          "message": "task(comparison_dashboard): Update the banner (#3050)\n\n# Change Summary\n\nRather than remove, I thought we might want to update the banner with\nsome new text and a link to file issues for feedback.\n\nI know the name \"Dataflow Engine\" is up for some debate, though we\nalready use this name elsewhere in the site.\n\nOpen to suggestions on all fronts including just removing the banner!\n\n## What issue does this PR close?\n\n* Closes #3019\n\n## How are these changes tested?\n\n<img width=\"2435\" height=\"817\" alt=\"image\"\nsrc=\"https://github.com/user-attachments/assets/b3cb0454-37b5-4afd-81f0-7dc6acac0136\"\n/>\n\n## Are there any user-facing changes?\n\nYes - Banner update.\n\nCo-authored-by: Cijo Thomas <cijo.thomas@gmail.com>",
          "timestamp": "2026-05-20T16:17:47Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/fabbe70cbf95113006d75b3e89725cd930d747c3"
        },
        "date": 1779303403677,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.41862952709198,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 99.29598292623214,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.31714152555537,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 26.833463541666667,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 27.37890625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 622604.4166017467,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 613771.9663662295,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.005999,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 16050593.51948151,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 16059542.664478812,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 26.150743922873037,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.1377045065164566,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.21716162317178,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.57854165052305,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 46.175,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 49.83203125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 2638391.078421646,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 2642024.26176506,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002477,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 20278200.25721903,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 20248760.400380548,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 7.675251340679117,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
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
          "id": "84dfec92b99db44ed2e74a980b1f40df5f4b3ee9",
          "message": "Update one_collect digest to 6ccba44 (#2979)\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n|---|---|---|---|\n| one_collect | workspace.dependencies | digest | `cfe3f78` → `6ccba44`\n|\n\n---\n\n### Configuration\n\n📅 **Schedule**: (UTC)\n\n- Branch creation\n  - At any time (no schedule defined)\n- Automerge\n  - At any time (no schedule defined)\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My4xNzkuMyIsInVwZGF0ZWRJblZlciI6IjQzLjE4NS4xIiwidGFyZ2V0QnJhbmNoIjoibWFpbiIsImxhYmVscyI6WyJkZXBlbmRlbmNpZXMiXX0=-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2026-05-20T22:51:33Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/84dfec92b99db44ed2e74a980b1f40df5f4b3ee9"
        },
        "date": 1779331421900,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.2776719033718109,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.20471192492484,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.52404898465355,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 27.081380208333332,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 27.9140625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 612203.4986161675,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 613903.4157735379,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002924,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 16076801.711742569,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 16066059.943812124,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 26.187835575870324,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.3966286182403564,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.22083710171508,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.60508820798515,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 46.186328125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 47.859375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 2634772.28114566,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 2597974.297675154,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003288,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 20153825.617769387,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 20169684.053886328,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 7.75751539797927,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
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
          "id": "32abb25dd613ea36a97504ba79da5e427a3bef72",
          "message": "Add AaronRM as Triager (#3063)\n\n# Change Summary\n\nUpdate docs\n\n## What issue does this PR close?\n\n* Closes #3062",
          "timestamp": "2026-05-22T15:56:41Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/32abb25dd613ea36a97504ba79da5e427a3bef72"
        },
        "date": 1779474349523,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.19888247549533844,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.19592983731947,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.59342960846398,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 46.4078125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 48.1171875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 2639616.875322255,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 2644866.6105606076,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003026,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 20258769.394208238,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 20233415.09908997,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 7.659656374849912,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.4386119842529297,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.17943375393612,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.46634508348795,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 26.825,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 27.3125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 607045.3087892191,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 598312.2827128533,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002111,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 15964939.416248154,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 15950774.580763612,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 26.683288773314676,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
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
          "id": "32abb25dd613ea36a97504ba79da5e427a3bef72",
          "message": "Add AaronRM as Triager (#3063)\n\n# Change Summary\n\nUpdate docs\n\n## What issue does this PR close?\n\n* Closes #3062",
          "timestamp": "2026-05-22T15:56:41Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/32abb25dd613ea36a97504ba79da5e427a3bef72"
        },
        "date": 1779507453451,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.3663489818572998,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.16944508272437,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.46457173862491,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 26.481640625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 27.265625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 617181.8370383298,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 608748.9795992513,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003386,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 15986009.191481799,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 15969471.969649894,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 26.260428727134183,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.5935678482055664,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.22231281101209,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.56189338092652,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 48.664453125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 53.83984375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 2642670.766151876,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 2600558.018076418,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.005583,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 20275119.752010066,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 20234791.25592343,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 7.796449689289061,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
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
          "id": "32abb25dd613ea36a97504ba79da5e427a3bef72",
          "message": "Add AaronRM as Triager (#3063)\n\n# Change Summary\n\nUpdate docs\n\n## What issue does this PR close?\n\n* Closes #3062",
          "timestamp": "2026-05-22T15:56:41Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/32abb25dd613ea36a97504ba79da5e427a3bef72"
        },
        "date": 1779557617951,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.24709071218967438,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 99.60739323658999,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.31763119138051,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 26.497395833333332,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 27.1171875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 627273.1213467503,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 625723.1877614044,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002571,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 16347083.412590155,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 16354128.512011576,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 26.12510409127349,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.7600680589675903,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.21467599947418,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.61676657866127,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 51.56302083333333,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 55.0859375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 2644279.0704836613,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 2664377.3912689597,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.005013,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 20431528.02010762,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 20395327.142060388,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 7.6684061676325515,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
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
          "id": "32abb25dd613ea36a97504ba79da5e427a3bef72",
          "message": "Add AaronRM as Triager (#3063)\n\n# Change Summary\n\nUpdate docs\n\n## What issue does this PR close?\n\n* Closes #3062",
          "timestamp": "2026-05-22T15:56:41Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/32abb25dd613ea36a97504ba79da5e427a3bef72"
        },
        "date": 1779589175293,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.2034589052200317,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.18050134305383,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.42714319011732,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 25.912630208333333,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 26.46484375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 605177.1545624152,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 597894.0961645364,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002265,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 15914672.495248826,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 15917980.51071475,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 26.61787864663781,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.4896560907363892,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.23721315793127,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.64541783162473,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 47.284244791666666,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 47.98828125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 2650371.54532577,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 2610890.1255252496,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002908,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 20360596.393729124,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 20329612.449379556,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 7.79833520938881,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
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
          "id": "32abb25dd613ea36a97504ba79da5e427a3bef72",
          "message": "Add AaronRM as Triager (#3063)\n\n# Change Summary\n\nUpdate docs\n\n## What issue does this PR close?\n\n* Closes #3062",
          "timestamp": "2026-05-22T15:56:41Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/32abb25dd613ea36a97504ba79da5e427a3bef72"
        },
        "date": 1779644048916,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.07332373410463333,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.16895624270941,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.46135729779981,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 26.396354166666665,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 27.2890625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 613680.6333854764,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 614130.6069645277,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003523,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 16085501.568990495,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 16074787.499618301,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 26.192313795426248,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.141831874847412,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.19766828762164,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.60286815728605,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 51.53763020833333,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 53.90625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 2600944.312859811,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 2571245.9002399654,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003207,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 20084677.343099456,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 20057366.97334088,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 7.811262758348015,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
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
          "id": "dae17d6eedbb715a81942cb8616a761ec45eeae3",
          "message": "perf: Box inner raw_batch_store record batch slices (#3077)\n\n# Change Summary\n\nBoxes raw_batch_store record batch slices. Attached issue explains the\nrationale, but basically, we have very a very large enum variant for\notap metrics (almost 1k) which is penalizing a lot of data structures.\n\nThis penalty is more visible when queues are larger and/or saturated,\nwhen batches are smaller in size, or when there are many processing\nstages.\n\n## What issue does this PR close?\n\n* Closes #3076\n\n## How are these changes tested?\n\nAd-hoc perf testing and also the manual pipelineperf run for the\nstandard continuous bench set.\n\n## Are there any user-facing changes?\n\nNo.",
          "timestamp": "2026-05-24T17:52:44Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/dae17d6eedbb715a81942cb8616a761ec45eeae3"
        },
        "date": 1779675706289,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.1817761659622192,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.18183347295788,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.40173872689304,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 26.208203125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 27.3125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 613447.323975416,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 606197.7495254362,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003522,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 16125948.469418976,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 16119586.534217352,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 26.601795341607957,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.49708425998687744,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.26843438615674,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.7005050348567,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 49.77734375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 62.94140625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 2672166.138519353,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 2685449.054990491,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001883,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 20546960.102769908,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 20516456.239176955,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 7.651219472805327,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
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
          "id": "dae17d6eedbb715a81942cb8616a761ec45eeae3",
          "message": "perf: Box inner raw_batch_store record batch slices (#3077)\n\n# Change Summary\n\nBoxes raw_batch_store record batch slices. Attached issue explains the\nrationale, but basically, we have very a very large enum variant for\notap metrics (almost 1k) which is penalizing a lot of data structures.\n\nThis penalty is more visible when queues are larger and/or saturated,\nwhen batches are smaller in size, or when there are many processing\nstages.\n\n## What issue does this PR close?\n\n* Closes #3076\n\n## How are these changes tested?\n\nAd-hoc perf testing and also the manual pipelineperf run for the\nstandard continuous bench set.\n\n## Are there any user-facing changes?\n\nNo.",
          "timestamp": "2026-05-24T17:52:44Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/dae17d6eedbb715a81942cb8616a761ec45eeae3"
        },
        "date": 1779733620365,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.31997305154800415,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.16448045440667,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.42774598743893,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 26.7375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 27.1484375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 593764.4928833256,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 595664.3792701196,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003588,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 15643948.794452233,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 15609293.436609708,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 26.263025520547494,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.7494043707847595,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.22990689184863,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.60819837272375,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 44.870052083333334,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 47.140625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 2693156.412814655,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 2672973.780827783,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002085,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 20454859.44239578,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 20428497.52143699,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 7.652472908305593,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
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
          "id": "b163b4888f8faef824a428dae32f4c82c041048b",
          "message": "[Attributes Processor] Add attribute update action (#3084)\n\n# Change Summary\n\nAdds support for the attributes processor `update` action.\n\n`update` replaces existing attribute values for matching keys without\ninserting missing attributes. This provides the generic primitive needed\nfor redaction-style replacements while preserving existing-only\nsemantics.\n\nThe implementation reuses the existing attribute mutation path and\nhandles transport-optimized attribute batches by materializing\n`parent_id` before value changes that can alter equality runs.\n\n\n## What issue does this PR close?\n\n* Closes #3054 \n\n## How are these changes tested?\n\n - `cargo +1.95 fmt --all`\n  - `cargo +1.95 check -p otap-df-pdata`\n\n## Are there any user-facing changes?\n\nYes. Users can configure a new attributes processor `update` action to\nreplace existing attribute values for matching keys without inserting\nmissing attributes.\n\n```yaml\nprocessors:\n  attributes/update:\n    actions:\n      - action: update\n         key: secret\n         value: \"[MASKED]\"\n```\n\n---------\n\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-05-25T20:20:10Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/b163b4888f8faef824a428dae32f4c82c041048b"
        },
        "date": 1779761834395,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.3978286981582642,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.17857482495396,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.46400371430781,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 26.16328125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 26.85546875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 603293.6208503502,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 594860.6096580444,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002292,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 15616813.373384356,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 15581506.624746244,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 26.25289541756964,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.3803991377353668,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.20290912840528,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.61219704585879,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 45.78111979166667,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 52.01171875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 2668116.505237521,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 2678265.9974245955,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003002,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 20537534.525039732,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 20512846.725603994,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 7.66822061168999,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
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
          "id": "99fda2c577da533a1840085d37ef4d0445c43d3f",
          "message": "Update dependency kubernetes to v36 (#3082)\n\nThis PR contains the following updates:\n\n| Package | Change |\n[Age](https://docs.renovatebot.com/merge-confidence/) |\n[Confidence](https://docs.renovatebot.com/merge-confidence/) |\n|---|---|---|---|\n| [kubernetes](https://redirect.github.com/kubernetes-client/python) |\n`==35.0.0` → `==36.0.0` |\n![age](https://developer.mend.io/api/mc/badges/age/pypi/kubernetes/36.0.0?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/pypi/kubernetes/35.0.0/36.0.0?slim=true)\n|\n\n---\n\n### Release Notes\n\n<details>\n<summary>kubernetes-client/python (kubernetes)</summary>\n\n###\n[`v36.0.0`](https://redirect.github.com/kubernetes-client/python/blob/HEAD/CHANGELOG.md#v3600)\n\n[Compare\nSource](https://redirect.github.com/kubernetes-client/python/compare/v35.0.0...v36.0.0)\n\nKubernetes API Version: v1.36.1\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: (UTC)\n\n- Branch creation\n  - \"before 8am on Monday\"\n- Automerge\n  - At any time (no schedule defined)\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My4xOTQuMCIsInVwZGF0ZWRJblZlciI6IjQzLjE5NC4wIiwidGFyZ2V0QnJhbmNoIjoibWFpbiIsImxhYmVscyI6WyJkZXBlbmRlbmNpZXMiXX0=-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2026-05-26T16:33:01Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/99fda2c577da533a1840085d37ef4d0445c43d3f"
        },
        "date": 1779819763079,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.29080232977867126,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.17095641116885,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.44959466790668,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 26.310546875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 27.28515625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 601750.3674423703,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 603500.2715767887,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003287,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 15813929.598393708,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 15805857.791618632,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 26.203682654650738,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.6018259525299072,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.25034226426726,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.57022723759468,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 47.83841145833333,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 49.84375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 2683408.2274386217,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 2699557.675127511,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002052,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 20653319.662222717,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 20626703.59091651,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 7.650631009855042,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
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
          "id": "0012b9b79d793aa9412b20f86f79094f99a9590d",
          "message": "fix(security): Revert tidy workflow changes and try different approach (#3098)\n\n# Change Summary\n\nReverts #3056\n\nWhile the intention was correct, the change above made CodeQL and\nOpenSSF very unhappy and flagged the `checkout` in `tidy-commit` as a\nDangerous workflow.\n\nInstead, trying to copy what the opentelemetry-collector maintainers did\nin https://github.com/open-telemetry/opentelemetry-collector/pull/15357.\n\nThis is safe because under the `pull_request` trigger ([GitHub\ndocs](https://docs.github.com/en/actions/writing-workflows/choosing-when-your-workflow-runs/events-that-trigger-workflows#pull_request)):\n\n- **Fork PRs cannot access secrets**, and the `GITHUB_TOKEN` is\nread-only — regardless of what `permissions:` the workflow requests. See\n[Workflows in forked\nrepositories](https://docs.github.com/en/actions/writing-workflows/choosing-when-your-workflow-runs/events-that-trigger-workflows#workflows-in-forked-repositories).\n- **Same-repo PRs** get write access, but the job is gated to\n`renovate[bot]` / `dependabot[bot]` actors and explicitly requires\n`github.event.pull_request.head.repo.fork == false`.",
          "timestamp": "2026-05-27T01:05:04Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/0012b9b79d793aa9412b20f86f79094f99a9590d"
        },
        "date": 1779848543530,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.16440916061401367,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.14831002072067,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.56163315425941,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 50.244921875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 54.0546875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 2676120.5649686744,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 2680520.352165634,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002902,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 20570760.25888822,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 20529047.385830063,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 7.674166787157121,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.26348984241485596,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.17942276580128,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.51593609430742,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 25.5671875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 26.32421875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 594498.3002597862,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 596064.7428729445,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.008582,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 15631285.266982565,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 15597063.750119252,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 26.22414000137312,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
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
          "id": "f80c48c28097c749fdc0f0b40a58dce4e2bf1034",
          "message": "docs: improve logging macro guidance and event naming references (#3086)\n\nImprove the otel_* logging macro documentation to be accurate and\nconsistent across docs.\n\n- **telemetry README**: Document that event names must follow OTel Event\nnaming conventions (with link to events guide). Note that target maps to\nInstrumentationScope.name and is auto-set to crate name. Replace the\noutdated example with a real call from the codebase.\n- **events-guide**: Add that target becomes InstrumentationScope.name in\nOTLP export. Link the event naming section anchor to the semantic\nconventions guide.",
          "timestamp": "2026-05-27T16:38:49Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/f80c48c28097c749fdc0f0b40a58dce4e2bf1034"
        },
        "date": 1779906871727,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.138787865638733,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.25516239510064,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.60185758513931,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 49.084635416666664,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 51.13671875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 2703040.074289854,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 2672258.1825751164,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00281,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 20470268.484958697,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 20451725.430469453,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 7.660288447590256,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.33998003602027893,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.18399524713402,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.43710854575036,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 27.123697916666668,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 27.796875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 617646.4186657111,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 615546.5441682102,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003586,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 16089727.893194774,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 16043412.449619683,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 26.138929778148405,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
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
          "id": "7e9b8f342c6717d08f85d0fc9ab3275f595bdac1",
          "message": "fix(comparison_dashboard): Fix landing page backpressure detection for a comparison (#3116)\n\n# Change Summary\n\nPull out backpressure detection for a comparison to a helper - There was\nalready a helper for backpressure detection for a test, but not for an\nentire comparison which determines when the warning sign is displayed in\nthe legend.\n\n## What issue does this PR close?\n\n* Closes #3109\n\n## How are these changes tested?\n\n<img width=\"2333\" height=\"712\" alt=\"image\"\nsrc=\"https://github.com/user-attachments/assets/03a1335d-3d13-4275-b08a-0f299ee703d5\"\n/>\n\n## Are there any user-facing changes?\n\nNo",
          "timestamp": "2026-05-27T21:18:42Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/7e9b8f342c6717d08f85d0fc9ab3275f595bdac1"
        },
        "date": 1779936401283,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.064648151397705,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.23322718655517,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.66631165833013,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 47.23268229166667,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 50.53515625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 2686211.1107277954,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 2657612.411968589,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00273,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 20383484.840799216,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 20342262.90132277,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 7.669848601324238,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.296913743019104,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.20727090608423,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.51210901207803,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 26.217578125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 26.79296875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 611824.6194753688,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 613641.2107837726,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002489,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 16097211.234956091,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 16046834.272786107,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 26.232285172627087,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
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
          "id": "70292bb27def0ed7a179f4cf46ff305dcebd096a",
          "message": "Update one_collect digest to 293b7d3 (#3114)\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n|---|---|---|---|\n| one_collect | workspace.dependencies | digest | `6ccba44` → `293b7d3`\n|\n\n---\n\n### Configuration\n\n📅 **Schedule**: (UTC)\n\n- Branch creation\n  - At any time (no schedule defined)\n- Automerge\n  - At any time (no schedule defined)\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My4xOTQuMCIsInVwZGF0ZWRJblZlciI6IjQzLjE5OC4wIiwidGFyZ2V0QnJhbmNoIjoibWFpbiIsImxhYmVscyI6WyJkZXBlbmRlbmNpZXMiXX0=-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-05-28T17:05:14Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/70292bb27def0ed7a179f4cf46ff305dcebd096a"
        },
        "date": 1779992844318,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.457582712173462,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.18232568627681,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.47617777088342,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 27.300260416666667,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 27.828125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 610514.4974446328,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 601615.7437150927,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.008403,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 16050890.846483545,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 15988259.28243309,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 26.679638979136772,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.39315706491470337,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.25323437231759,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.55592380362397,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 51.44192708333333,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 53.3984375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 2678807.2600259464,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 2668275.3397252536,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00805,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 20369730.673526153,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 20452101.16760326,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 7.6340437473831235,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
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
          "id": "7d0af33f05ddaf772194302fa68db3f5b9100c64",
          "message": "chore(deps): update one_collect digest to f655a30 (#3130)\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n|---|---|---|---|\n| one_collect | workspace.dependencies | digest | `293b7d3` → `f655a30`\n|\n\n---\n\n### Configuration\n\n📅 **Schedule**: (UTC)\n\n- Branch creation\n  - At any time (no schedule defined)\n- Automerge\n  - At any time (no schedule defined)\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My4xOTguMCIsInVwZGF0ZWRJblZlciI6IjQzLjE5OC4wIiwidGFyZ2V0QnJhbmNoIjoibWFpbiIsImxhYmVscyI6WyJkZXBlbmRlbmNpZXMiXX0=-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2026-05-28T23:19:33Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/7d0af33f05ddaf772194302fa68db3f5b9100c64"
        },
        "date": 1780021407837,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.4298451244831085,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.19976027566224,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.53932723608743,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 44.258854166666666,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 45.3046875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 2663428.7904878673,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 2674877.40978438,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.007236,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 20509706.98003043,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 20492345.03663904,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 7.667531567992008,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 2.755110263824463,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.16762420543306,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.50287482593222,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 26.063932291666667,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 26.79296875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 618761.867684646,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 601714.295827131,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.008546,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 16019371.596436612,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 15981650.630917778,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 26.622886821088397,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
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
          "id": "d624d457aedfdfa11eb8b3d7db98fbc57576a6a8",
          "message": "  chore: document component naming conventions in AGENTS.md (#3125)\n\n# Change Summary\n\nDocument naming conventions for new OTAP Dataflow components in\n`AGENTS.md`, covering consistent module names, component URNs, and\ntelemetry metric set names.\n\n  ## What issue does this PR close?\n\n  None.\n\n  ## How are these changes tested?\n\n  - `npx markdownlint-cli rust/otap-dataflow/AGENTS.md`\n  - `python3 tools/sanitycheck.py`\n  - `git diff --check`\n\n  ## Are there any user-facing changes?\n\n  No. This is contributor/agent guidance only.\n\n  ### Changelog\n\n* [x] Added a `.chloggen/*.yaml` entry, OR this PR is a `chore`\n(indicated in title).\n\n---------\n\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-05-29T15:25:20Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/d624d457aedfdfa11eb8b3d7db98fbc57576a6a8"
        },
        "date": 1780079744231,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.3139221668243408,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.2264067022278,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.65235144115601,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 48.450390625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 59.0703125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 2724305.6961619253,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 2688510.439033495,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00795,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 20578862.75310861,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 20539347.24009286,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 7.654373386218505,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.4237695932388306,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.17378599450741,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.47295831396251,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 26.104427083333334,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 26.75,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 609860.7989768111,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 601177.7864807895,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002217,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 16007308.177659584,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 15970538.165252188,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 26.626579586987273,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
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
          "id": "ddeb7d3c651e6fdb1abda29a396062b41b32389c",
          "message": "feat(host metrics receiver) - Add opt-in host load average metrics (#3141)\n\n# Change Summary\n\nAdds opt-in Linux load average metrics to the Rust `host_metrics`\nreceiver.\n\nWhen `families.load.enabled: true`, the receiver reads `/proc/loadavg`\nthrough the existing `host_view.root_path` abstraction and emits the\nCollector-compatible gauges:\n\n  - `system.cpu.load_average.1m`\n  - `system.cpu.load_average.5m`\n  - `system.cpu.load_average.15m`\n\nThe load family defaults to disabled because these metric names are\ndevelopment/experimental and are not registered in Semantic Conventions\n1.41.0.\n\n  ## What issue does this PR close?\n\n  * Closes #3067\n\n  ## How are these changes tested?\n\n- `cargo check -p otap-df-core-nodes --features otap-df-otap/crypto-ring\n--all-targets`\n- `cargo test -p otap-df-core-nodes --features otap-df-otap/crypto-ring\nhost_metrics_receiver` on Linux test VM\n  - `cargo xtask check` on Linux test VM\n- `npx markdownlint-cli\nrust/otap-dataflow/crates/core-nodes/src/receivers/host_metrics_receiver/README.md`\n  - `python3 tools/sanitycheck.py`\n\n  ## Are there any user-facing changes?\n\n  Yes. Users can now enable Linux load average metrics with:\n\n  ```yaml\n  families:\n    load:\n      enabled: true\n      interval: 30s\n```\n\n  ### Changelog\n\n  - [x] Added a .chloggen/*.yaml entry, OR this PR is a chore (indicated in title).",
          "timestamp": "2026-05-29T23:21:41Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/ddeb7d3c651e6fdb1abda29a396062b41b32389c"
        },
        "date": 1780109737767,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.8178429007530212,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.20141389068155,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.53581828048685,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.576953125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.09765625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1889701.3322658823,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1874246.5448341058,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.095812,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 20629754.21470721,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 20591317.04203336,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.006958647765948,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 2.346262216567993,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 76.9352035973512,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 81.37565042225148,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 23.918229166666666,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 24.32421875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 384772.6298916709,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 375744.8552387228,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00327,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 10990398.358476352,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 10957967.953638459,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 29.249630980293258,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
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
          "id": "ddeb7d3c651e6fdb1abda29a396062b41b32389c",
          "message": "feat(host metrics receiver) - Add opt-in host load average metrics (#3141)\n\n# Change Summary\n\nAdds opt-in Linux load average metrics to the Rust `host_metrics`\nreceiver.\n\nWhen `families.load.enabled: true`, the receiver reads `/proc/loadavg`\nthrough the existing `host_view.root_path` abstraction and emits the\nCollector-compatible gauges:\n\n  - `system.cpu.load_average.1m`\n  - `system.cpu.load_average.5m`\n  - `system.cpu.load_average.15m`\n\nThe load family defaults to disabled because these metric names are\ndevelopment/experimental and are not registered in Semantic Conventions\n1.41.0.\n\n  ## What issue does this PR close?\n\n  * Closes #3067\n\n  ## How are these changes tested?\n\n- `cargo check -p otap-df-core-nodes --features otap-df-otap/crypto-ring\n--all-targets`\n- `cargo test -p otap-df-core-nodes --features otap-df-otap/crypto-ring\nhost_metrics_receiver` on Linux test VM\n  - `cargo xtask check` on Linux test VM\n- `npx markdownlint-cli\nrust/otap-dataflow/crates/core-nodes/src/receivers/host_metrics_receiver/README.md`\n  - `python3 tools/sanitycheck.py`\n\n  ## Are there any user-facing changes?\n\n  Yes. Users can now enable Linux load average metrics with:\n\n  ```yaml\n  families:\n    load:\n      enabled: true\n      interval: 30s\n```\n\n  ### Changelog\n\n  - [x] Added a .chloggen/*.yaml entry, OR this PR is a chore (indicated in title).",
          "timestamp": "2026-05-29T23:21:41Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/ddeb7d3c651e6fdb1abda29a396062b41b32389c"
        },
        "date": 1780163282705,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.07772061228752136,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.16647967017924,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.47129386303911,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 23.219791666666666,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.890625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 548954.2652052484,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 549380.9158085198,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002259,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 15777265.768531227,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 15752578.10757995,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 28.718263256946127,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.7758827805519104,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.21798119163701,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.58277480827329,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 40.860026041666664,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.5546875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1881544.6100750691,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1896143.1908482239,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.008025,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 20807324.05020275,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 20758943.113893397,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 10.973498283584146,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
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
          "id": "ddeb7d3c651e6fdb1abda29a396062b41b32389c",
          "message": "feat(host metrics receiver) - Add opt-in host load average metrics (#3141)\n\n# Change Summary\n\nAdds opt-in Linux load average metrics to the Rust `host_metrics`\nreceiver.\n\nWhen `families.load.enabled: true`, the receiver reads `/proc/loadavg`\nthrough the existing `host_view.root_path` abstraction and emits the\nCollector-compatible gauges:\n\n  - `system.cpu.load_average.1m`\n  - `system.cpu.load_average.5m`\n  - `system.cpu.load_average.15m`\n\nThe load family defaults to disabled because these metric names are\ndevelopment/experimental and are not registered in Semantic Conventions\n1.41.0.\n\n  ## What issue does this PR close?\n\n  * Closes #3067\n\n  ## How are these changes tested?\n\n- `cargo check -p otap-df-core-nodes --features otap-df-otap/crypto-ring\n--all-targets`\n- `cargo test -p otap-df-core-nodes --features otap-df-otap/crypto-ring\nhost_metrics_receiver` on Linux test VM\n  - `cargo xtask check` on Linux test VM\n- `npx markdownlint-cli\nrust/otap-dataflow/crates/core-nodes/src/receivers/host_metrics_receiver/README.md`\n  - `python3 tools/sanitycheck.py`\n\n  ## Are there any user-facing changes?\n\n  Yes. Users can now enable Linux load average metrics with:\n\n  ```yaml\n  families:\n    load:\n      enabled: true\n      interval: 30s\n```\n\n  ### Changelog\n\n  - [x] Added a .chloggen/*.yaml entry, OR this PR is a chore (indicated in title).",
          "timestamp": "2026-05-29T23:21:41Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/ddeb7d3c651e6fdb1abda29a396062b41b32389c"
        },
        "date": 1780195026173,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.588141679763794,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.2007968605079,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.51387214258038,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 40.63294270833333,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.62109375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1877384.3918630069,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1888426.0726714549,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002459,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 20718156.487419747,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 20661106.601727597,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 10.971123936088684,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.4518500566482544,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.20283376668021,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.57904761904761,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 23.112760416666667,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 24.21484375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 557116.6261930146,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 549028.1282508591,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.008175,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 15781064.425005382,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 15770939.960121956,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 28.74363554975234,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
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
          "id": "ddeb7d3c651e6fdb1abda29a396062b41b32389c",
          "message": "feat(host metrics receiver) - Add opt-in host load average metrics (#3141)\n\n# Change Summary\n\nAdds opt-in Linux load average metrics to the Rust `host_metrics`\nreceiver.\n\nWhen `families.load.enabled: true`, the receiver reads `/proc/loadavg`\nthrough the existing `host_view.root_path` abstraction and emits the\nCollector-compatible gauges:\n\n  - `system.cpu.load_average.1m`\n  - `system.cpu.load_average.5m`\n  - `system.cpu.load_average.15m`\n\nThe load family defaults to disabled because these metric names are\ndevelopment/experimental and are not registered in Semantic Conventions\n1.41.0.\n\n  ## What issue does this PR close?\n\n  * Closes #3067\n\n  ## How are these changes tested?\n\n- `cargo check -p otap-df-core-nodes --features otap-df-otap/crypto-ring\n--all-targets`\n- `cargo test -p otap-df-core-nodes --features otap-df-otap/crypto-ring\nhost_metrics_receiver` on Linux test VM\n  - `cargo xtask check` on Linux test VM\n- `npx markdownlint-cli\nrust/otap-dataflow/crates/core-nodes/src/receivers/host_metrics_receiver/README.md`\n  - `python3 tools/sanitycheck.py`\n\n  ## Are there any user-facing changes?\n\n  Yes. Users can now enable Linux load average metrics with:\n\n  ```yaml\n  families:\n    load:\n      enabled: true\n      interval: 30s\n```\n\n  ### Changelog\n\n  - [x] Added a .chloggen/*.yaml entry, OR this PR is a chore (indicated in title).",
          "timestamp": "2026-05-29T23:21:41Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/ddeb7d3c651e6fdb1abda29a396062b41b32389c"
        },
        "date": 1780249964501,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.23022836446762085,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.1997018766463,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.46298162648269,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 23.717578125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 24.6171875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 548535.9556336859,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 549798.8409773762,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00228,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 15796363.890054945,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 15778628.021158732,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 28.731169862005864,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.1343623846769333,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.18678535711571,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.47563546187229,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 40.80677083333333,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.89453125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1886071.6953211327,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1888605.8663165472,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.005422,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 20686371.234676972,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 20649138.133256894,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 10.953249486099898,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
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
          "id": "74ec7428dc1d6fcf58fe69f19cfbb98eb6002ae6",
          "message": "chore(deps): update docker digest updates (#3147)\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n|---|---|---|---|\n| docker.io/rust | stage | digest | `a9cfb75` → `f49565f` |\n| gcr.io/distroless/cc-debian13 | final | digest | `8f960b7` → `e1fd250`\n|\n| golang | stage | digest | `b54cbf5` → `2d6c802` |\n| python | final | digest | `5b3879b` → `c845af9` |\n\n---\n\n### Configuration\n\n📅 **Schedule**: (UTC)\n\n- Branch creation\n  - \"before 8am on the first day of the month\"\n- Automerge\n  - At any time (no schedule defined)\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n👻 **Immortal**: This PR will be recreated if closed unmerged. Get\n[config\nhelp](https://redirect.github.com/renovatebot/renovate/discussions) if\nthat's undesired.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My4yMDIuMSIsInVwZGF0ZWRJblZlciI6IjQzLjIwMi4xIiwidGFyZ2V0QnJhbmNoIjoibWFpbiIsImxhYmVscyI6WyJkZXBlbmRlbmNpZXMiXX0=-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2026-06-01T01:15:06Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/74ec7428dc1d6fcf58fe69f19cfbb98eb6002ae6"
        },
        "date": 1780281576861,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.23484034836292267,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.16960345846873,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.434122929246,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 22.678645833333334,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.19140625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 548653.1884321265,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 549941.6475287416,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003457,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 15833317.731895741,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 15817563.299847964,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 28.790905004277285,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.5138098001480103,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.21133849352918,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.5283458588363,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 41.1515625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 42.17578125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1879954.469619364,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1889613.8594004007,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002134,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 20743653.646893553,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 20689201.879705057,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 10.977720947429857,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
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
          "id": "43bb12cc2b86b6bbbdfbfd48431609a186a4b52b",
          "message": "Add OTAP-ATTR-OTAP saturation scaling tests (1,2,4,8 cores) (#3145)\n\n## Summary\n\nAdd OTAP saturation/scaling tests to measure OTAP throughput scaling\nacross core counts, matching the existing OTLP scaling tests.\n\n**Config:** OTAP receiver → attribute rename processor → OTAP exporter,\nzstd compression, batch size 512, 1KB synthetic logs.\n\n**Core layouts (NUMA-aware):**\n| Cores | SUT | Loadgen | Backend |\n|-------|-----|---------|---------|\n| 1 | 0 (1) | 32-41 (10) | 42-43 (2) |\n| 2 | 0-1 (2) | 32-51 (20) | 52-55 (4) |\n| 4 | 0-3 (4) | 32-63,96-119 (56) | 120-127 (8) |\n| 8 | 0-7 (8) | 32-63,96-127 (64) | 64-79 (16) |\n\n## Results (3 runs, highly consistent ±2%)\n\n| Cores | Throughput | SUT CPU | LG CPU (allocated) | BE CPU (allocated)\n| Scaling |\n|-------|-----------|---------|-------|-------|---------|\n| 1 | **2.47M** logs/sec | 100% | 10.0/10 ✅ | 0.63/2 | 100% |\n| 2 | **4.82M** logs/sec | 100% | 20.0/20 ✅ | 0.71/4 | 97% |\n| 4 | **9.04M** logs/sec | 100% ✅ | 56.0/56 ✅ | 2.35/8 | 92% |\n| 8 | **14.1M** logs/sec | 92% ⚠️ | 64.0/64 ⚠️ | 2.87/16 | 72% |\n\n### Analysis\n- **1-4 cores: fully saturated** — near-linear scaling (92-100%)\n- **8 cores: loadgen bottleneck** — used all 64 allocated cores, SUT\nonly at 92%. True throughput is higher.\n- **Backend is never the bottleneck** — peaks at ~3 cores out of 16\nallocated\n- **Per-core throughput: ~2.4-2.5M logs/sec** (vs OTLP ~120K — **~20x\nfaster**)\n\n### 8-core loadgen limitation\n\nThe 8-core SUT cannot be fully saturated due to a NUMA topology\nconstraint:\n- **CI machine:** 2-socket Intel Xeon 8358, 2 NUMA nodes × 64 logical\ncores (32 physical + 32 HT)\n- **SUT on NUMA0** (cores 0-7), **Loadgen on NUMA1** (cores 32-63,\n96-127 = 64 cores)\n- Placing loadgen on the same NUMA node as SUT causes significant\nthroughput reduction (tested: 14M → 8.7M)\n- 64 cores is the maximum loadgen allocation without cross-NUMA\ncontention\n- Each loadgen core produces ~220K logs/sec → 64 × 220K ≈ 14M,\ninsufficient for 8 × 2.5M = 20M theoretical max\n\n### Comparison with OTLP scaling (same test)\n| Cores | OTLP | OTAP | Speedup |\n|-------|------|------|---------|\n| 1 | 121K | 2.47M | 20.4x |\n| 2 | 264K | 4.82M | 18.3x |\n| 4 | 567K | 9.04M | 15.9x |\n| 8 | 1.03M | 14.1M | 13.7x |",
          "timestamp": "2026-06-01T18:19:04Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/43bb12cc2b86b6bbbdfbfd48431609a186a4b52b"
        },
        "date": 1780342150154,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.11352682113647461,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 80.61939748780947,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.48360688643558,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 23.712369791666667,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 24.40625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 398355.49565533607,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 397903.25529398373,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00349,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11510169.918995123,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11480007.0701023,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 28.92705642855583,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.610499382019043,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.21864224511393,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.56985246156236,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.978515625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.0625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1889233.6572831958,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1858807.5604099317,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.007434,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 20745602.21728998,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 20690074.53085326,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.160704668489112,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
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
          "id": "b31a3c75011eccdc455a74fcc4a9838eefc5a6da",
          "message": "fix(deps): update module go.opentelemetry.io/collector/pdata to v1.59.0 (#3162)\n\nThis PR contains the following updates:\n\n| Package | Change |\n[Age](https://docs.renovatebot.com/merge-confidence/) |\n[Confidence](https://docs.renovatebot.com/merge-confidence/) |\n|---|---|---|---|\n|\n[go.opentelemetry.io/collector/pdata](https://redirect.github.com/open-telemetry/opentelemetry-collector)\n| `v1.58.0` → `v1.59.0` |\n![age](https://developer.mend.io/api/mc/badges/age/go/go.opentelemetry.io%2fcollector%2fpdata/v1.59.0?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/go/go.opentelemetry.io%2fcollector%2fpdata/v1.58.0/v1.59.0?slim=true)\n|\n\n---\n\n### Release Notes\n\n<details>\n<summary>open-telemetry/opentelemetry-collector\n(go.opentelemetry.io/collector/pdata)</summary>\n\n###\n[`v1.59.0`](https://redirect.github.com/open-telemetry/opentelemetry-collector/blob/HEAD/CHANGELOG.md#v1590v01530)\n\n##### 🛑 Breaking changes 🛑\n\n- `pkg/configoptional`: Stabilize feature gate\nconfigoptional.AddEnabledField\n([#&#8203;15333](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/15333))\n- `pkg/confmap`: Stabilize confmap.newExpandedValueSanitizer feature\ngate\n([#&#8203;15339](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/15339))\n- `pkg/exporterhelper`: mark exporter.PersistRequestContext as stable\n([#&#8203;15330](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/15330))\n- `pkg/otelcol`: Stabilize otelcol.printInitialConfig gate\n([#&#8203;15340](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/15340))\n- `pkg/pdata`: Remove pdata.useCustomProtoEncoding feature gate\n([#&#8203;15332](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/15332))\n- `pkg/service`: Stabilize telemetry.UseLocalHostAsDefaultMetricsAddress\ngate\n([#&#8203;15342](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/15342))\n- `pkg/xpdata`: Stabilize pdata.enableRefCounting feature gate\n([#&#8203;15331](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/15331))\n\n##### 🧰 Bug fixes 🧰\n\n- `pkg/config/configgrpc`: Fix memory corruption and fatal error in\nSnappy\n([#&#8203;15237](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/15237),\n[#&#8203;15320](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/15320))\n\n<!-- previous-version -->\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: (UTC)\n\n- Branch creation\n  - \"before 8am on Monday\"\n- Automerge\n  - At any time (no schedule defined)\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My4yMDYuMSIsInVwZGF0ZWRJblZlciI6IjQzLjIwNi4xIiwidGFyZ2V0QnJhbmNoIjoibWFpbiIsImxhYmVscyI6WyJkZXBlbmRlbmNpZXMiXX0=-->\n\n---------\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: otelbot <197425009+otelbot@users.noreply.github.com>",
          "timestamp": "2026-06-01T23:18:45Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/b31a3c75011eccdc455a74fcc4a9838eefc5a6da"
        },
        "date": 1780370325844,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.500265121459961,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.17089276246902,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.47078637770899,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 23.054036458333332,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.80078125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 547084.2496337312,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 538876.5354615649,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.009887,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 15767386.515745286,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 15754931.755542265,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 29.259738508079625,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.1274659633636475,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.21098785148304,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.56732691710648,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 40.020703125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.0625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1877662.9773796215,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1856492.967487011,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003373,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 20717967.30625833,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 20671437.829894423,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.159733793283696,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
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
          "id": "39d17768106ad32f00b474b9c75b6b8fd740206b",
          "message": "df_engine: MVP weaver live-check for in-tree semconv events (#1613) (#3047)\n\nInitial wiring for #1613: validates one internal event\n(`tls.handshake.failed`) end-to-end via `weaver registry live-check` in\nCI. Establishes the registry → emit → live-check mechanism so subsequent\nPRs can backfill the remaining events and tighten gates.\n\nBuilds on #3049 (event_name / InstrumentationScope.name encoder change),\nwhich is already merged.\n\n## Changes\n\n- New in-tree semconv registry at `rust/otap-dataflow/semconv/` with one\n`type: event` group. `manifest.yaml` pulls upstream\n`semantic-conventions@v1.41.0` via `dependencies:` (no second checkout\nin CI).\n- New `configs/internal-events-otlp.yaml` wires `internal_telemetry` →\n`otlp_grpc` and a TLS-enabled `otlp` receiver whose handshakes are\nfailed by plaintext HTTP from CI.\n- New workflow `.github/workflows/df-engine-internal-observability.yml`\nholding the live-check job. Kept out of `rust-ci.yml` to convey\nlong-term intent and avoid polluting the rust workflow as the registry\ngrows. **Not in required status checks yet**, mirroring the staged\nrollout used for the host-metrics live-check. The assert step is\nregistry-driven: it discovers every declared event and fails if any\nreceived zero samples or has event-level violations.\n\n## Verified locally\n\n`weaver registry check` clean; `cargo xtask check` green; end-to-end\nsmoke produced `tls.handshake.failed` samples with 0 event-level\nviolations.\n\n## Deferred (follow-ups under #1613)\n\nBackfill remaining event names; attribute-level alignment (e.g. `error`\n→ `error.type` per OTel semconv); `InstrumentationScope.version`; xtask\nstatic drift check; promote workflow to required. Severity declaration\nin semconv is blocked on open-telemetry/weaver#1004 (the wire already\ncarries `severity_number = 13`).\n\nFollow-up: adopt `weaver-live-check-{start,stop}` composite actions once\nopen-telemetry/weaver#1448 merges to drop ~half the workflow\nboilerplate.",
          "timestamp": "2026-06-02T17:58:58Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/39d17768106ad32f00b474b9c75b6b8fd740206b"
        },
        "date": 1780427544406,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.28072479367256165,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.1584694069441,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.55734514649879,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.83984375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 40.73828125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1872264.0686418486,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1867008.1594477321,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.007125,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 20841958.3860545,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 20794264.03619383,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.163292608329911,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.1573919653892517,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.18937728714388,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.53513480012396,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 22.473046875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 22.9921875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 547561.5322245839,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 548423.3500979953,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003397,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 15789840.145726144,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 15775578.09371641,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 28.7913345463951,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
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
          "id": "9fb01d6d441d84fcdb5a0a75d377f041fa0cdfca",
          "message": "Add uncompressed bytes-per-log metric to traffic generator benchmarks (#3026)\n\nCloses #2987\n\nAdds a `logs_bytes_produced` counter metric to the traffic generator\nreceiver that tracks the total protobuf-encoded (uncompressed) bytes of\nlog payloads produced. The benchmark report SQL then computes\n`uncompressed_bytes_per_log` from this counter, enabling direct\ncomparison of uncompressed payload size against the egress (compressed)\nbytes per log.\n\n### Changes\n- **metrics.rs**: Added `logs_bytes_produced: Counter<u64>` with unit\n`By`\n- **mod.rs**: Record payload bytes in `export_pdata()` for log signals\n(captured before ownership move)\n- **integration_report_logs.yaml** & **report_logs.yaml**: Added\n`logs_bytes_produced` to metric filter and `uncompressed_bytes_per_log`\ncomputed metric to report SQL",
          "timestamp": "2026-06-02T22:17:51Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/9fb01d6d441d84fcdb5a0a75d377f041fa0cdfca"
        },
        "date": 1780455988077,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.9887781739234924,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.17625386046123,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.43904119152897,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 23.054427083333334,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.5703125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 543633.6781220911,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 538258.347082039,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.007467,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 15746759.073704187,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 15727122.13575142,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 29.255020677466863,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.4104715883731842,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.19721822858456,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.68086654845425,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 40.495052083333334,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.1875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1870843.867487981,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1878523.1499870196,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.005606,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 20620581.980094668,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 20556534.57103334,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 10.977017760061756,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
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
          "id": "a7a04541634b883904108e499e52bbeb94ccfb6e",
          "message": "feat: added comment support in OPL (#3152)\n\n# Change Summary\n\n<!--\nReplace with a brief summary of the change in this PR\n-->\n\nAdds support for comments in OPL programs. Both inline comments (`//`)\nand block comments (`/* ... */`) are supported\n\n## What issue does this PR close?\n\n<!--\nWe highly recommend correlation of every PR to an issue\n-->\n\n* Closes https://github.com/open-telemetry/otel-arrow/issues/3151\n\n## How are these changes tested?\n\nUnit\n\n## Are there any user-facing changes?\n\nYes - this comment syntax is now available for OPL programs written in\nthe transform processor config.\n\n <!-- If yes, provide further info below -->\n\n### Changelog\n\n<!--\nUser-facing changes need a .chloggen/*.yaml entry. Copy the\nTEMPLATE.yaml\nin go/.chloggen/ or rust/otap-dataflow/.chloggen/ and fill in the\nfields.\nIf not required, include `chore` in the PR title.\n-->\n\n* [x] Added a `.chloggen/*.yaml` entry, OR this PR is a `chore`\n(indicated in title).",
          "timestamp": "2026-06-03T18:14:18Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/a7a04541634b883904108e499e52bbeb94ccfb6e"
        },
        "date": 1780514079764,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.2910943329334259,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 77.96830908983847,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 81.46344517628826,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 24.6609375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 25.16015625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 375205.2022609376,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 376297.40334092936,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003603,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 10841744.192067249,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 10809086.313456275,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 28.811637007881544,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.4913570284843445,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.2048719660442,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.57937089419585,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 40.90846354166667,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 42.234375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1880775.0125171207,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1890016.332505125,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001818,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 20743126.55941454,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 20698484.777820107,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 10.97510439601362,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
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
          "id": "ee0b8953fa04dee164e3f9149751559be8b05f2e",
          "message": "task(comparison_dashboard): Add comparisons to fluent bit (#3190)\n\n# Change Summary\n\nThis PR adds comparisons to fluent bit for logs across all comparisons.\nMetrics and traces are not included though we may want to add them in\nthe future as I think there is some support there.\n\n## What issue does this PR close?\n\n* Closes #3169\n\n## How are these changes tested?\n\n<img width=\"2335\" height=\"1572\" alt=\"image\"\nsrc=\"https://github.com/user-attachments/assets/17be789e-6aca-40da-aefb-b0fac929da7d\"\n/>\n\n## Are there any user-facing changes?\n\nYes new comparisons on the dashboard.\n\n### Changelog\n\n<!--\nUser-facing changes need a .chloggen/*.yaml entry. Copy the\nTEMPLATE.yaml\nin go/.chloggen/ or rust/otap-dataflow/.chloggen/ and fill in the\nfields.\nIf not required, include `chore` in the PR title.\n-->\n\n* [x] Added a `.chloggen/*.yaml` entry, OR this PR is a `chore`\n(indicated in title).",
          "timestamp": "2026-06-04T01:18:21Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/ee0b8953fa04dee164e3f9149751559be8b05f2e"
        },
        "date": 1780543785906,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.8746676445007324,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 81.1091201356243,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.31840498442368,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 23.877083333333335,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 25.078125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 388270.43292598845,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 384874.3572319783,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003374,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11494821.774454704,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11465276.82807461,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 29.866426688246055,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.522068440914154,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.22886492587433,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.58305336426913,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 41.14609375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 42.21875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1856754.5230507804,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1866448.0520595198,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00209,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 21231581.572437182,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 21167562.253201928,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.375393785543258,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
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
          "id": "c6ed105cab28e537bf5c2c81a97e9b63677d3cff",
          "message": "fix(comparison_dashboard): Remove fluent bit otlpgrpc gzip/zstd suites (#3200)\n\n# Change Summary\n\nFluent bit seems to ignore compression settings on the otlp grpc path,\nso removing those suites.",
          "timestamp": "2026-06-04T03:45:28Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/c6ed105cab28e537bf5c2c81a97e9b63677d3cff"
        },
        "date": 1780600556090,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.6050196886062622,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.18608684743884,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.54971083029692,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 40.20065104166667,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.03125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1864398.6690216528,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1834474.7043336458,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004883,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 21187243.38457586,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 21123963.951327726,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.549487891288154,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.09196459501981735,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 77.558262007085,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 79.60981449727201,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 23.26171875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 24.12890625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 371135.8260122159,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 371477.1395664288,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003477,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11051228.054590305,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11017630.264671026,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 29.749416256108777,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
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
          "id": "c6ed105cab28e537bf5c2c81a97e9b63677d3cff",
          "message": "fix(comparison_dashboard): Remove fluent bit otlpgrpc gzip/zstd suites (#3200)\n\n# Change Summary\n\nFluent bit seems to ignore compression settings on the otlp grpc path,\nso removing those suites.",
          "timestamp": "2026-06-04T03:45:28Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/c6ed105cab28e537bf5c2c81a97e9b63677d3cff"
        },
        "date": 1780628324951,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 3.014326810836792,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 85.61001047846761,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.53028358902836,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 24.752734375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 26.00390625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 435323.66918943485,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 422201.5908480329,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.010006,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 12746458.486667449,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 12721220.219791472,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 30.190455846139635,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.7859750390052795,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.22400845486055,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.48699047176387,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 41.15234375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 42.0390625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1879247.4746006813,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1864477.0584174334,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003184,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 21186428.554764643,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 21117614.5755694,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.36320152565871,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Tom Tan",
            "username": "ThomsonTan",
            "email": "Tom.Tan@microsoft.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "082af67b453a42301bdb6f62fd00b1640b641d40",
          "message": "fix(otap): Transform processor config error on leading whitespace (#3219)\n\n# Change Summary\n\nIn `SignalScope::try_from`\n(`rust/otap-dataflow/crates/core-nodes/src/processors/transform_processor/mod.rs`),\nthe query slice is now trimmed of leading whitespace before the keyword\nchecks.\n\n## What issue does this PR close?\n\n<!--\nWe highly recommend correlation of every PR to an issue\n-->\n\n* Closes #3209 \n\n## How are these changes tested?\n\nAdded test and passed locally.\n\n## Are there any user-facing changes?\n\n <!-- If yes, provide further info below -->\n\n### Changelog\n\n<!--\nUser-facing changes need a .chloggen/*.yaml entry. Copy the\nTEMPLATE.yaml\nin go/.chloggen/ or rust/otap-dataflow/.chloggen/ and fill in the\nfields.\nIf not required, include `chore` in the PR title.\n-->\n\n* [x] Added a `.chloggen/*.yaml` entry, OR this PR is a `chore`\n(indicated in title).",
          "timestamp": "2026-06-05T16:49:35Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/082af67b453a42301bdb6f62fd00b1640b641d40"
        },
        "date": 1780685379922,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.004610738251358271,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 76.95642850454063,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 77.9774810224632,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 23.808203125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 24.28125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 370128.3588562301,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 370111.29320582293,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003573,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 10969527.637129374,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 10939066.180068543,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 29.6384569682101,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.4663408994674683,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.23775032149341,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.54809143742736,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 40.82265625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.4921875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1843524.3062754297,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1816491.9569721243,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002776,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 20962362.86008472,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 20894788.63648208,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.54002514551536,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
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
          "id": "f9b6074475f5c7916343ca39bf98b2879f053775",
          "message": "fix: replace OPL `exclude`/`date_time` keywords to match spec (#3180)\n\n# Change Summary\n\n<!--\nReplace with a brief summary of the change in this PR\n-->\n\nIn https://github.com/open-telemetry/otel-arrow/pull/3051 we are adding\na specification for OPL. It has some minor keyword differences from the\ncurrent implementation.\n\nThe keyword for the operation it specifies to remove attributes is\n`remove`, but currently we use `exclude`. I think `remove` is a more\nsensible name, so we'll make this change. (`project-away` will remain an\nalias`).\n\n```\n// before this would be `exclude attributes[\"x\"]`\nlogs | remove attributes[\"x\"]\n```\n\nThe tag we use for timestamp literals also changes to match the spec,\nfrom `date_time` to `timestamp`. This seems more sensible as well,\nbecause that is what the arrow type is called. E.g., now\ntimestamp/datetime literals will be defined like `timestamp\"...\"`\n\n## What issue does this PR close?\n\n<!--\nWe highly recommend correlation of every PR to an issue\n-->\n\n* Relates to #3051 \n\n## How are these changes tested?\n\nUnit tests\n\n## Are there any user-facing changes?\n\nYes - this is a breaking change to OPL syntax\n <!-- If yes, provide further info below -->\n\n### Changelog\n\n<!--\nUser-facing changes need a .chloggen/*.yaml entry. Copy the\nTEMPLATE.yaml\nin go/.chloggen/ or rust/otap-dataflow/.chloggen/ and fill in the\nfields.\nIf not required, include `chore` in the PR title.\n-->\n\n* [x] Added a `.chloggen/*.yaml` entry, OR this PR is a `chore`\n(indicated in title).",
          "timestamp": "2026-06-06T00:22:55Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/f9b6074475f5c7916343ca39bf98b2879f053775"
        },
        "date": 1780713755714,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.48666834831237793,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.22244296215352,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.57307286821707,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 41.8125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 42.3125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1856781.0269399034,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1865817.3919895324,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002888,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 21211084.38795564,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 21145444.33526396,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.368253120064516,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.28229212760925293,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.19037859492803,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.4950495049505,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 22.948177083333334,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.7265625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 535028.9334400716,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 536539.2781111118,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002198,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 15891757.553729508,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 15878834.571351768,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 29.61900125872702,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
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
          "id": "f9b6074475f5c7916343ca39bf98b2879f053775",
          "message": "fix: replace OPL `exclude`/`date_time` keywords to match spec (#3180)\n\n# Change Summary\n\n<!--\nReplace with a brief summary of the change in this PR\n-->\n\nIn https://github.com/open-telemetry/otel-arrow/pull/3051 we are adding\na specification for OPL. It has some minor keyword differences from the\ncurrent implementation.\n\nThe keyword for the operation it specifies to remove attributes is\n`remove`, but currently we use `exclude`. I think `remove` is a more\nsensible name, so we'll make this change. (`project-away` will remain an\nalias`).\n\n```\n// before this would be `exclude attributes[\"x\"]`\nlogs | remove attributes[\"x\"]\n```\n\nThe tag we use for timestamp literals also changes to match the spec,\nfrom `date_time` to `timestamp`. This seems more sensible as well,\nbecause that is what the arrow type is called. E.g., now\ntimestamp/datetime literals will be defined like `timestamp\"...\"`\n\n## What issue does this PR close?\n\n<!--\nWe highly recommend correlation of every PR to an issue\n-->\n\n* Relates to #3051 \n\n## How are these changes tested?\n\nUnit tests\n\n## Are there any user-facing changes?\n\nYes - this is a breaking change to OPL syntax\n <!-- If yes, provide further info below -->\n\n### Changelog\n\n<!--\nUser-facing changes need a .chloggen/*.yaml entry. Copy the\nTEMPLATE.yaml\nin go/.chloggen/ or rust/otap-dataflow/.chloggen/ and fill in the\nfields.\nIf not required, include `chore` in the PR title.\n-->\n\n* [x] Added a `.chloggen/*.yaml` entry, OR this PR is a `chore`\n(indicated in title).",
          "timestamp": "2026-06-06T00:22:55Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/f9b6074475f5c7916343ca39bf98b2879f053775"
        },
        "date": 1780768721809,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.8037542700767517,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.22804571799885,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.66126880719715,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 42.06471354166667,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 42.82421875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1841929.01419898,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1827124.4314579917,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003042,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 21100631.590164196,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 21031737.768675447,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.548546572346204,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.0813707113265991,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.17605190421577,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.42888494482598,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 22.958463541666667,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.59375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 541316.9126719873,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 535463.2697120425,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002293,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 15878597.75133177,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 15861579.535292037,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 29.65394388278181,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
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
          "id": "f9b6074475f5c7916343ca39bf98b2879f053775",
          "message": "fix: replace OPL `exclude`/`date_time` keywords to match spec (#3180)\n\n# Change Summary\n\n<!--\nReplace with a brief summary of the change in this PR\n-->\n\nIn https://github.com/open-telemetry/otel-arrow/pull/3051 we are adding\na specification for OPL. It has some minor keyword differences from the\ncurrent implementation.\n\nThe keyword for the operation it specifies to remove attributes is\n`remove`, but currently we use `exclude`. I think `remove` is a more\nsensible name, so we'll make this change. (`project-away` will remain an\nalias`).\n\n```\n// before this would be `exclude attributes[\"x\"]`\nlogs | remove attributes[\"x\"]\n```\n\nThe tag we use for timestamp literals also changes to match the spec,\nfrom `date_time` to `timestamp`. This seems more sensible as well,\nbecause that is what the arrow type is called. E.g., now\ntimestamp/datetime literals will be defined like `timestamp\"...\"`\n\n## What issue does this PR close?\n\n<!--\nWe highly recommend correlation of every PR to an issue\n-->\n\n* Relates to #3051 \n\n## How are these changes tested?\n\nUnit tests\n\n## Are there any user-facing changes?\n\nYes - this is a breaking change to OPL syntax\n <!-- If yes, provide further info below -->\n\n### Changelog\n\n<!--\nUser-facing changes need a .chloggen/*.yaml entry. Copy the\nTEMPLATE.yaml\nin go/.chloggen/ or rust/otap-dataflow/.chloggen/ and fill in the\nfields.\nIf not required, include `chore` in the PR title.\n-->\n\n* [x] Added a `.chloggen/*.yaml` entry, OR this PR is a `chore`\n(indicated in title).",
          "timestamp": "2026-06-06T00:22:55Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/f9b6074475f5c7916343ca39bf98b2879f053775"
        },
        "date": 1780800422745,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.2311806678771973,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.23193738379499,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.60645580388214,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 41.952213541666666,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 43.171875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1856728.4678688701,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1833868.7849453755,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002932,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 21239421.74088929,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 21185791.566389088,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.581756511288203,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.2654663026332855,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 77.99757711818557,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 81.6943444015444,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 23.867708333333333,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 24.39453125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 369641.7845287498,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 368660.51017000544,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003606,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 10954264.48665304,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 10923361.770380026,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 29.713691009654248,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
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
          "id": "f9b6074475f5c7916343ca39bf98b2879f053775",
          "message": "fix: replace OPL `exclude`/`date_time` keywords to match spec (#3180)\n\n# Change Summary\n\n<!--\nReplace with a brief summary of the change in this PR\n-->\n\nIn https://github.com/open-telemetry/otel-arrow/pull/3051 we are adding\na specification for OPL. It has some minor keyword differences from the\ncurrent implementation.\n\nThe keyword for the operation it specifies to remove attributes is\n`remove`, but currently we use `exclude`. I think `remove` is a more\nsensible name, so we'll make this change. (`project-away` will remain an\nalias`).\n\n```\n// before this would be `exclude attributes[\"x\"]`\nlogs | remove attributes[\"x\"]\n```\n\nThe tag we use for timestamp literals also changes to match the spec,\nfrom `date_time` to `timestamp`. This seems more sensible as well,\nbecause that is what the arrow type is called. E.g., now\ntimestamp/datetime literals will be defined like `timestamp\"...\"`\n\n## What issue does this PR close?\n\n<!--\nWe highly recommend correlation of every PR to an issue\n-->\n\n* Relates to #3051 \n\n## How are these changes tested?\n\nUnit tests\n\n## Are there any user-facing changes?\n\nYes - this is a breaking change to OPL syntax\n <!-- If yes, provide further info below -->\n\n### Changelog\n\n<!--\nUser-facing changes need a .chloggen/*.yaml entry. Copy the\nTEMPLATE.yaml\nin go/.chloggen/ or rust/otap-dataflow/.chloggen/ and fill in the\nfields.\nIf not required, include `chore` in the PR title.\n-->\n\n* [x] Added a `.chloggen/*.yaml` entry, OR this PR is a `chore`\n(indicated in title).",
          "timestamp": "2026-06-06T00:22:55Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/f9b6074475f5c7916343ca39bf98b2879f053775"
        },
        "date": 1780855472477,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.1386161595582962,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 77.03444260139514,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 78.60444513473635,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 24.66875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 25.1328125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 369348.552597427,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 369860.5293366171,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002726,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 10979173.605898643,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 10945679.956252774,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 29.68463173291543,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.4796294569969177,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.21069752174702,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.57120185974429,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 41.79453125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 43.078125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1871576.584500367,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1880553.2165484321,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002905,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 21346163.670637496,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 21297492.697943117,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.351002185312389,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
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
          "id": "f9b6074475f5c7916343ca39bf98b2879f053775",
          "message": "fix: replace OPL `exclude`/`date_time` keywords to match spec (#3180)\n\n# Change Summary\n\n<!--\nReplace with a brief summary of the change in this PR\n-->\n\nIn https://github.com/open-telemetry/otel-arrow/pull/3051 we are adding\na specification for OPL. It has some minor keyword differences from the\ncurrent implementation.\n\nThe keyword for the operation it specifies to remove attributes is\n`remove`, but currently we use `exclude`. I think `remove` is a more\nsensible name, so we'll make this change. (`project-away` will remain an\nalias`).\n\n```\n// before this would be `exclude attributes[\"x\"]`\nlogs | remove attributes[\"x\"]\n```\n\nThe tag we use for timestamp literals also changes to match the spec,\nfrom `date_time` to `timestamp`. This seems more sensible as well,\nbecause that is what the arrow type is called. E.g., now\ntimestamp/datetime literals will be defined like `timestamp\"...\"`\n\n## What issue does this PR close?\n\n<!--\nWe highly recommend correlation of every PR to an issue\n-->\n\n* Relates to #3051 \n\n## How are these changes tested?\n\nUnit tests\n\n## Are there any user-facing changes?\n\nYes - this is a breaking change to OPL syntax\n <!-- If yes, provide further info below -->\n\n### Changelog\n\n<!--\nUser-facing changes need a .chloggen/*.yaml entry. Copy the\nTEMPLATE.yaml\nin go/.chloggen/ or rust/otap-dataflow/.chloggen/ and fill in the\nfields.\nIf not required, include `chore` in the PR title.\n-->\n\n* [x] Added a `.chloggen/*.yaml` entry, OR this PR is a `chore`\n(indicated in title).",
          "timestamp": "2026-06-06T00:22:55Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/f9b6074475f5c7916343ca39bf98b2879f053775"
        },
        "date": 1780887015490,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.39334195852279663,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.20795382901342,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.46838889750349,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 23.432552083333334,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 24.34375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 533661.5333417463,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 535760.6481176864,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002436,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 15864430.531897157,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 15842146.851288132,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 29.611041026686866,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.084489107131958,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.22640164352362,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.61147286821705,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 42.24036458333333,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 43.23046875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1856877.1708257024,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1877014.8004044457,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00309,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 21255057.829244297,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 21200411.709079053,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.323862669950396,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
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
          "id": "66541e92c02992528ccb1693cf5ea2269a13b230",
          "message": "chore(ci): restore continuous/nightly pipeline perf workflows (#3364)\n\n## Root cause\n\nAll four pipeline perf workflows set `TMPDIR`/`TMP`/`TEMP` in job-level\n`env:` using `${{ runner.temp }}`. The `runner` context is not available\nat job-level `env` scope (only inside `steps`), so GitHub Actions\nrejects the workflow at evaluation time:\n\n> Unrecognized named-value: 'runner'. Located at position 1 within\nexpression: runner.temp\n\nFailing since #3164 introduced the pattern on June 8.\n\n## Fix\n\nReplace the job-level `env` block with a `Route temp files to\nRUNNER_TEMP` step at the top of `steps:` that writes\n`TMPDIR`/`TMP`/`TEMP` to `$GITHUB_ENV` using `$RUNNER_TEMP`. Placed\nbefore `harden-runner` so every subsequent step inherits the routed temp\ndirs.",
          "timestamp": "2026-07-08T11:04:53Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/66541e92c02992528ccb1693cf5ea2269a13b230"
        },
        "date": 1783533259353,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.2944433391094208,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.19764531975788,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.57377740173831,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 41.795963541666666,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 42.8203125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 2034382.8710180777,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 2040372.9756060864,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002959,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 23175935.140585084,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 23093335.225592416,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.358675799801134,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 3.0613410472869873,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 77.02991604015035,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 81.61832555244321,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 24.494921875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 25.07421875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 373225.17836275505,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 361799.48244314716,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002297,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 10894375.710468374,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 10859838.378530396,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 30.111639842327044,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
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
          "id": "66541e92c02992528ccb1693cf5ea2269a13b230",
          "message": "chore(ci): restore continuous/nightly pipeline perf workflows (#3364)\n\n## Root cause\n\nAll four pipeline perf workflows set `TMPDIR`/`TMP`/`TEMP` in job-level\n`env:` using `${{ runner.temp }}`. The `runner` context is not available\nat job-level `env` scope (only inside `steps`), so GitHub Actions\nrejects the workflow at evaluation time:\n\n> Unrecognized named-value: 'runner'. Located at position 1 within\nexpression: runner.temp\n\nFailing since #3164 introduced the pattern on June 8.\n\n## Fix\n\nReplace the job-level `env` block with a `Route temp files to\nRUNNER_TEMP` step at the top of `steps:` that writes\n`TMPDIR`/`TMP`/`TEMP` to `$GITHUB_ENV` using `$RUNNER_TEMP`. Placed\nbefore `harden-runner` so every subsequent step inherits the routed temp\ndirs.",
          "timestamp": "2026-07-08T11:04:53Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/66541e92c02992528ccb1693cf5ea2269a13b230"
        },
        "date": 1783566576648,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.5715466737747192,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 76.59955987198148,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 78.93270515304145,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 24.559244791666668,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 25.19921875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 371390.21055456204,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 365553.63983451517,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002357,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11028588.478946118,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 10995909.446837911,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 30.169549081603236,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.5971373319625854,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.21123310618154,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.48237770897832,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 45.25598958333333,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 51.19921875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 2053171.6258092513,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 2020379.654477161,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00298,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 23386308.425316013,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 23303042.7609003,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.5752048747333,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
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
          "id": "d2b3ed04a389abf5f9e460123d95ea42b30312bf",
          "message": "feat: add opamp controller extension (#3410)\n\n# Change Summary\n\nAdds initial implementation of OpAMP Agent Controller Extension.\n\nDesign doc can be found in #3388 \n\nAdds:\n- controller extension\n- OpAMP proto definitions & prost generated structs\n\nCurrently only works with websocket.\n\nFollowups will need to include:\n- [plain HTTP\ntransport](https://opentelemetry.io/docs/specs/opamp/#plain-http-transport)\n(plain meaning traditional HTTP request/response, not necessarily\nplaintext)\n- mTLS\n- connection setting management (this is ignored)\n- metrics collection\n\nCurrently this lives in the controller crate. I wasn't sure where was\nthe right place for this, and hesitated between controller or\ncore-nodes. Happy to move if anyone has feelings/suggestions\n\n<!--Replace with a brief summary of the change in this PR-->\n\n## What issue does this PR close?\n\n<!--We highly recommend correlation of every PR to an issue-->\n\n* Relates to #3387 \n\n## How are these changes tested?\n\n## Are there any user-facing changes?\n\n <!-- If yes, provide further info below -->\n\n### Changelog\n\n<!--\nUser-facing changes need a .chloggen/*.yaml entry. Copy the\nTEMPLATE.yaml\nin go/.chloggen/ or rust/otap-dataflow/.chloggen/ and fill in the\nfields.\nIf not required, include `chore` in the PR title.\n-->\n\n* [ ] Added a `.chloggen/*.yaml` entry\n* [ ] This PR is a `chore` (indicated in title)\n* [ ] This is a documentation-only PR.\n\n---------\n\nCo-authored-by: cijothomas <cijo.thomas@gmail.com>\nCo-authored-by: Lalit Kumar Bhasin <lalit_fin@yahoo.com>",
          "timestamp": "2026-07-09T17:01:29Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/d2b3ed04a389abf5f9e460123d95ea42b30312bf"
        },
        "date": 1783620325586,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 2.8124513626098633,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.22359573094963,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.51589249650459,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 23.740364583333335,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 24.48046875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 548188.7233666827,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 532771.1821507512,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.008531,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 16059407.38474315,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 16043941.480131498,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 30.14316074663181,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.03655892238020897,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.22699796835386,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.52839498481427,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 42.940104166666664,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 43.89453125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 2030589.1477958392,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 2031331.509316767,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00311,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 23041937.234584663,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 22974273.887795668,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.343267767423525,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
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
          "id": "d2b3ed04a389abf5f9e460123d95ea42b30312bf",
          "message": "feat: add opamp controller extension (#3410)\n\n# Change Summary\n\nAdds initial implementation of OpAMP Agent Controller Extension.\n\nDesign doc can be found in #3388 \n\nAdds:\n- controller extension\n- OpAMP proto definitions & prost generated structs\n\nCurrently only works with websocket.\n\nFollowups will need to include:\n- [plain HTTP\ntransport](https://opentelemetry.io/docs/specs/opamp/#plain-http-transport)\n(plain meaning traditional HTTP request/response, not necessarily\nplaintext)\n- mTLS\n- connection setting management (this is ignored)\n- metrics collection\n\nCurrently this lives in the controller crate. I wasn't sure where was\nthe right place for this, and hesitated between controller or\ncore-nodes. Happy to move if anyone has feelings/suggestions\n\n<!--Replace with a brief summary of the change in this PR-->\n\n## What issue does this PR close?\n\n<!--We highly recommend correlation of every PR to an issue-->\n\n* Relates to #3387 \n\n## How are these changes tested?\n\n## Are there any user-facing changes?\n\n <!-- If yes, provide further info below -->\n\n### Changelog\n\n<!--\nUser-facing changes need a .chloggen/*.yaml entry. Copy the\nTEMPLATE.yaml\nin go/.chloggen/ or rust/otap-dataflow/.chloggen/ and fill in the\nfields.\nIf not required, include `chore` in the PR title.\n-->\n\n* [ ] Added a `.chloggen/*.yaml` entry\n* [ ] This PR is a `chore` (indicated in title)\n* [ ] This is a documentation-only PR.\n\n---------\n\nCo-authored-by: cijothomas <cijo.thomas@gmail.com>\nCo-authored-by: Lalit Kumar Bhasin <lalit_fin@yahoo.com>",
          "timestamp": "2026-07-09T17:01:29Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/d2b3ed04a389abf5f9e460123d95ea42b30312bf"
        },
        "date": 1783649581560,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.540410041809082,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.20458072845261,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.51992231802997,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 23.153385416666666,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.76953125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 538987.0064560985,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 530684.3965094227,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002337,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 16023303.389513338,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 16004196.51881981,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 30.193658405837137,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.524505078792572,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.2202684864734,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.52267988536907,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 41.57630208333333,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 42.6640625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 2018924.160830168,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 2029513.5206194795,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002872,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 23043536.717503604,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 22976236.012255855,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.354216901432565,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
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
          "id": "021c7430e5267668d8257f337e2d122f51387597",
          "message": "feat(contrib-extensions): Add Azure Identity Auth extension (#3438)\n\n# Change Summary\n\nAdds a new `azure_identity_auth` extension (in a new\n`otap-df-contrib-extensions`\ncrate) that acquires and refreshes Azure OAuth access tokens and exposes\nthem\nto data-path nodes through the shared `BearerTokenProvider` capability\n(merged\nin #3372).\n\n## What issue does this PR close?\n\n* Related to #3356\n\n## How are these changes tested?\n\nUnit tests\n\n## Are there any user-facing changes?\n\nYes — a new opt-in extension is available. It is not enabled by default;\nusers\nmust build with `--features azure-identity-auth-extension` and reference\nthe\nURN `urn:microsoft:extension:azure_identity_auth` in their pipeline\nconfig.\n\n### Changelog\n\n* [x] Added a `.chloggen/*.yaml` entry\n* [ ] This PR is a `chore` (indicated in title)\n* [ ] This is a documentation-only PR.",
          "timestamp": "2026-07-10T16:07:59Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/021c7430e5267668d8257f337e2d122f51387597"
        },
        "date": 1783711119679,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.6586501598358154,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.18938013050678,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.47345319280842,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 23.189713541666666,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 24.1015625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 535050.7962011057,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 538574.9091577122,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002617,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 15991627.377969671,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 15977641.242740523,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 29.692484937665007,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.3166833817958832,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.22079210344495,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.60538330494037,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 42.42825520833333,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 43.265625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 2039708.598200981,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 2046168.0167453173,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002924,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 23271418.917194787,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 23207739.134934172,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.37317108211419,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
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
          "id": "59957e07eec5630be835293d6226289bcbfc0596",
          "message": "fix(pdata): Fix silent attribute loss when decoding OTAP batches whose root IDs are not monotonic in resource/scope visitation order (#3450)\n\n# Change Summary\n\nAddresses a follow-up from #2421.\n\nOn that PR review, it was noted that a 'truly pathological' OTAP batch\nmight drop attributes in certain cases if root IDs are not in expected\norder. However, with query-engine, it is easily possible to construct\nsuch a batch when adding attributes to a record which previously didn't\nhave any.\n\nThe OTLP decoder joined child records (attributes, datapoints, span\nevents/links) to parents with a shared forward-only cursor that assumed\nparents were visited in ascending ID order. When root IDs aren't\nmonotonic in `(resource_id, scope_id, id)` visitation order, a\nlater-visited smaller-ID record's child rows were skipped — silently\ndropping all of that record's attributes with no error.\n\n`ChildIndexIter::new` now binary-searches to each parent's rows\n(`SortedBatchCursor::seek_to_parent`), making the join order-independent\nacross logs, metrics, and traces. Adds regression tests in `pdata` and\n`query-engine`.\n\nThis could have a minor performance impact adding a `O(P * log n)`\nsearch where `P` is the number of parent records and `n` is the number\nof child rows.\n\n## What issue does this PR close?\n\n* Closes #3448 \n\n## How are these changes tested?\n\nUnit tests\n\n## Are there any user-facing changes?\n\n <!-- If yes, provide further info below -->\n\n### Changelog\n\n<!--\nUser-facing changes need a .chloggen/*.yaml entry. Copy the\nTEMPLATE.yaml\nin go/.chloggen/ or rust/otap-dataflow/.chloggen/ and fill in the\nfields.\nIf not required, include `chore` in the PR title.\n-->\n\n* [x] Added a `.chloggen/*.yaml` entry\n* [ ] This PR is a `chore` (indicated in title)\n* [ ] This is a documentation-only PR.",
          "timestamp": "2026-07-10T20:58:39Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/59957e07eec5630be835293d6226289bcbfc0596"
        },
        "date": 1783735271928,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.5028207302093506,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.18213711151375,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.40557445816827,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 23.4625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 24.19140625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 539965.7567030013,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 531851.0392581988,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003568,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 16049581.838652434,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 16031611.572994078,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 30.176836471049576,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.10785863548517227,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.19089217787092,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.59805729287315,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 42.27421875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 43.40625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 2041091.1009179405,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 2043292.5938153109,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002919,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 23209597.324157786,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 23125813.964329556,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.358920104937088,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
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
          "id": "59957e07eec5630be835293d6226289bcbfc0596",
          "message": "fix(pdata): Fix silent attribute loss when decoding OTAP batches whose root IDs are not monotonic in resource/scope visitation order (#3450)\n\n# Change Summary\n\nAddresses a follow-up from #2421.\n\nOn that PR review, it was noted that a 'truly pathological' OTAP batch\nmight drop attributes in certain cases if root IDs are not in expected\norder. However, with query-engine, it is easily possible to construct\nsuch a batch when adding attributes to a record which previously didn't\nhave any.\n\nThe OTLP decoder joined child records (attributes, datapoints, span\nevents/links) to parents with a shared forward-only cursor that assumed\nparents were visited in ascending ID order. When root IDs aren't\nmonotonic in `(resource_id, scope_id, id)` visitation order, a\nlater-visited smaller-ID record's child rows were skipped — silently\ndropping all of that record's attributes with no error.\n\n`ChildIndexIter::new` now binary-searches to each parent's rows\n(`SortedBatchCursor::seek_to_parent`), making the join order-independent\nacross logs, metrics, and traces. Adds regression tests in `pdata` and\n`query-engine`.\n\nThis could have a minor performance impact adding a `O(P * log n)`\nsearch where `P` is the number of parent records and `n` is the number\nof child rows.\n\n## What issue does this PR close?\n\n* Closes #3448 \n\n## How are these changes tested?\n\nUnit tests\n\n## Are there any user-facing changes?\n\n <!-- If yes, provide further info below -->\n\n### Changelog\n\n<!--\nUser-facing changes need a .chloggen/*.yaml entry. Copy the\nTEMPLATE.yaml\nin go/.chloggen/ or rust/otap-dataflow/.chloggen/ and fill in the\nfields.\nIf not required, include `chore` in the PR title.\n-->\n\n* [x] Added a `.chloggen/*.yaml` entry\n* [ ] This PR is a `chore` (indicated in title)\n* [ ] This is a documentation-only PR.",
          "timestamp": "2026-07-10T20:58:39Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/59957e07eec5630be835293d6226289bcbfc0596"
        },
        "date": 1783791154974,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.4977318346500397,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.24089145746022,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.52369278510473,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 42.55716145833333,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 43.84375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 2031516.3879797761,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 2041627.8921790202,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002942,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 23191397.392742876,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 23125399.398711648,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.359267514704063,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.19108682870864868,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.19483257701675,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.4467429631921,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 24.195833333333333,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 25.015625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 540327.0658230656,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 541359.5596635096,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002295,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 16053857.025433348,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 16041679.066140775,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 29.65470312450356,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
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
          "id": "59957e07eec5630be835293d6226289bcbfc0596",
          "message": "fix(pdata): Fix silent attribute loss when decoding OTAP batches whose root IDs are not monotonic in resource/scope visitation order (#3450)\n\n# Change Summary\n\nAddresses a follow-up from #2421.\n\nOn that PR review, it was noted that a 'truly pathological' OTAP batch\nmight drop attributes in certain cases if root IDs are not in expected\norder. However, with query-engine, it is easily possible to construct\nsuch a batch when adding attributes to a record which previously didn't\nhave any.\n\nThe OTLP decoder joined child records (attributes, datapoints, span\nevents/links) to parents with a shared forward-only cursor that assumed\nparents were visited in ascending ID order. When root IDs aren't\nmonotonic in `(resource_id, scope_id, id)` visitation order, a\nlater-visited smaller-ID record's child rows were skipped — silently\ndropping all of that record's attributes with no error.\n\n`ChildIndexIter::new` now binary-searches to each parent's rows\n(`SortedBatchCursor::seek_to_parent`), making the join order-independent\nacross logs, metrics, and traces. Adds regression tests in `pdata` and\n`query-engine`.\n\nThis could have a minor performance impact adding a `O(P * log n)`\nsearch where `P` is the number of parent records and `n` is the number\nof child rows.\n\n## What issue does this PR close?\n\n* Closes #3448 \n\n## How are these changes tested?\n\nUnit tests\n\n## Are there any user-facing changes?\n\n <!-- If yes, provide further info below -->\n\n### Changelog\n\n<!--\nUser-facing changes need a .chloggen/*.yaml entry. Copy the\nTEMPLATE.yaml\nin go/.chloggen/ or rust/otap-dataflow/.chloggen/ and fill in the\nfields.\nIf not required, include `chore` in the PR title.\n-->\n\n* [x] Added a `.chloggen/*.yaml` entry\n* [ ] This PR is a `chore` (indicated in title)\n* [ ] This is a documentation-only PR.",
          "timestamp": "2026-07-10T20:58:39Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/59957e07eec5630be835293d6226289bcbfc0596"
        },
        "date": 1783821141707,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.227487564086914,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.1762371980249,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.5231889672271,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 42.521484375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 43.51171875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 2003953.1657296517,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1979354.8901865913,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.008109,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 22868225.937227856,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 22800841.211655404,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.553373298848948,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.3022009134292603,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.18168550579594,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.45291582700358,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 23.930989583333332,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 24.58203125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 547810.2846689929,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 540676.693834505,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00232,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 16039563.3763326,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 16024838.419821043,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 29.665719938063628,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
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
          "id": "59957e07eec5630be835293d6226289bcbfc0596",
          "message": "fix(pdata): Fix silent attribute loss when decoding OTAP batches whose root IDs are not monotonic in resource/scope visitation order (#3450)\n\n# Change Summary\n\nAddresses a follow-up from #2421.\n\nOn that PR review, it was noted that a 'truly pathological' OTAP batch\nmight drop attributes in certain cases if root IDs are not in expected\norder. However, with query-engine, it is easily possible to construct\nsuch a batch when adding attributes to a record which previously didn't\nhave any.\n\nThe OTLP decoder joined child records (attributes, datapoints, span\nevents/links) to parents with a shared forward-only cursor that assumed\nparents were visited in ascending ID order. When root IDs aren't\nmonotonic in `(resource_id, scope_id, id)` visitation order, a\nlater-visited smaller-ID record's child rows were skipped — silently\ndropping all of that record's attributes with no error.\n\n`ChildIndexIter::new` now binary-searches to each parent's rows\n(`SortedBatchCursor::seek_to_parent`), making the join order-independent\nacross logs, metrics, and traces. Adds regression tests in `pdata` and\n`query-engine`.\n\nThis could have a minor performance impact adding a `O(P * log n)`\nsearch where `P` is the number of parent records and `n` is the number\nof child rows.\n\n## What issue does this PR close?\n\n* Closes #3448 \n\n## How are these changes tested?\n\nUnit tests\n\n## Are there any user-facing changes?\n\n <!-- If yes, provide further info below -->\n\n### Changelog\n\n<!--\nUser-facing changes need a .chloggen/*.yaml entry. Copy the\nTEMPLATE.yaml\nin go/.chloggen/ or rust/otap-dataflow/.chloggen/ and fill in the\nfields.\nIf not required, include `chore` in the PR title.\n-->\n\n* [x] Added a `.chloggen/*.yaml` entry\n* [ ] This PR is a `chore` (indicated in title)\n* [ ] This is a documentation-only PR.",
          "timestamp": "2026-07-10T20:58:39Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/59957e07eec5630be835293d6226289bcbfc0596"
        },
        "date": 1783877673964,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 2.6663851737976074,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.19976228723802,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.54850578281456,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 42.6328125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 43.64453125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 2060579.4944519764,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 2005636.5102673532,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003439,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 23179406.4561274,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 23101378.548164126,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.557132280683087,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.3803577423095703,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 77.13389018840654,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 81.19178944089705,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 24.18984375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 24.76171875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 376467.7083456419,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 371271.10746206576,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002299,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 10998793.716461863,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 10966099.51084379,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 29.624696065490777,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
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
          "id": "59957e07eec5630be835293d6226289bcbfc0596",
          "message": "fix(pdata): Fix silent attribute loss when decoding OTAP batches whose root IDs are not monotonic in resource/scope visitation order (#3450)\n\n# Change Summary\n\nAddresses a follow-up from #2421.\n\nOn that PR review, it was noted that a 'truly pathological' OTAP batch\nmight drop attributes in certain cases if root IDs are not in expected\norder. However, with query-engine, it is easily possible to construct\nsuch a batch when adding attributes to a record which previously didn't\nhave any.\n\nThe OTLP decoder joined child records (attributes, datapoints, span\nevents/links) to parents with a shared forward-only cursor that assumed\nparents were visited in ascending ID order. When root IDs aren't\nmonotonic in `(resource_id, scope_id, id)` visitation order, a\nlater-visited smaller-ID record's child rows were skipped — silently\ndropping all of that record's attributes with no error.\n\n`ChildIndexIter::new` now binary-searches to each parent's rows\n(`SortedBatchCursor::seek_to_parent`), making the join order-independent\nacross logs, metrics, and traces. Adds regression tests in `pdata` and\n`query-engine`.\n\nThis could have a minor performance impact adding a `O(P * log n)`\nsearch where `P` is the number of parent records and `n` is the number\nof child rows.\n\n## What issue does this PR close?\n\n* Closes #3448 \n\n## How are these changes tested?\n\nUnit tests\n\n## Are there any user-facing changes?\n\n <!-- If yes, provide further info below -->\n\n### Changelog\n\n<!--\nUser-facing changes need a .chloggen/*.yaml entry. Copy the\nTEMPLATE.yaml\nin go/.chloggen/ or rust/otap-dataflow/.chloggen/ and fill in the\nfields.\nIf not required, include `chore` in the PR title.\n-->\n\n* [x] Added a `.chloggen/*.yaml` entry\n* [ ] This PR is a `chore` (indicated in title)\n* [ ] This is a documentation-only PR.",
          "timestamp": "2026-07-10T20:58:39Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/59957e07eec5630be835293d6226289bcbfc0596"
        },
        "date": 1783907545815,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.42556604743003845,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.20949400802131,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.5348769005171,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 43.08450520833333,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 49.30078125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 2022948.7317545644,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 2031557.7140669627,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.008022,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 23079905.89492103,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 23009389.035134852,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.360694178221259,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.0316725969314575,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 77.59078831515669,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 81.58492564042669,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 24.721875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 25.19140625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 377986.8291627682,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 374087.2427138578,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00226,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11084604.478478882,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11049455.94671903,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 29.631067870864502,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
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
          "id": "035e910605e727284dd65a849142868778665c89",
          "message": "chore: Optimize per-event field extraction in the ETW receiver (#3427)\n\n## Summary\n\nCaches per-schema field readers instead of rebuilding them every event\nand drops a redundant per-field copy.\n\n## Problem\nPer event, `extract_decoded_fields` called\n`EventFormat::try_get_field_data_closure(name)` once for every field.\nEach call does two costly things:\n- Finds the field by name by re-walking the field list from index 0.\nReading all n fields is therefore `1 + 2 + ... + n = O(n^2)` name\ncomparisons per event.\n- Heap-allocates a boxed closure (`Box<dyn FnMut>`) for the result, so\n`n` allocations per event.\n\nThe result was then `.to_vec()`'d before interpretation, adding another\n`n` copies. So, a single event with `n` fields cost `O(n^2)`\nname-finding + `~2n` allocations, repeated on every event on the decode\nthread.\n\n### Performance\n\nCaching field readers per schema (instead of rebuilding a boxed closure\nper field on every event) turns per-event field extraction from an\nO(n^2) name-walk + n allocations into an O(n) reuse. A microbenchmark\nreading 16 fixed-size fields:\n\n| | time (16 fields) |\n|---|---|\n| `closure_per_field` (before) | ~990 ns |\n| `cached_refs` (after) | ~72 ns |\n\n~14x faster. The gap compounds two effects the change removes: the\nO(n^2) re-walk to find each field by name, and a heap allocation (boxed\nclosure) per field per event.\n\n<details>\n<summary>Benchmark used to measure this (criterion, against\none_collect's public API)</summary>\n\n```rust\nuse criterion::{black_box, criterion_group, criterion_main, Criterion};\nuse one_collect::event::{EventField, EventFormat, LocationType};\n\nconst FIELD_COUNT: usize = 16;\n\nfn bench_field_reads(c: &mut Criterion) {\n    // All-fixed-size schema (the common TraceLogging shape after struct\n    // flattening), so absolute offsets are valid for the cached-reader path.\n    let mut format = EventFormat::new();\n    let mut names = Vec::with_capacity(FIELD_COUNT);\n    for i in 0..FIELD_COUNT {\n        let name = format!(\"f{i}\");\n        format.add_field(EventField::new(\n            name.clone(),\n            \"u32\".to_string(),\n            LocationType::Static,\n            i * 4,\n            4,\n        ));\n        names.push(name);\n    }\n\n    let data = vec![0xABu8; FIELD_COUNT * 4];\n    let data = data.as_slice();\n\n    // Readers resolved once (a consumer caches these per schema_id).\n    let refs: Vec<_> = names\n        .iter()\n        .map(|n| format.get_field_ref(n).expect(\"field exists\"))\n        .collect();\n\n    let mut group = c.benchmark_group(\"tdh_field_reads\");\n\n    // Before: a fresh boxed closure per field, per event (re-walks the field\n    // list each call, so reading all n fields is O(n^2) plus n allocations).\n    group.bench_function(\"closure_per_field\", |b| {\n        b.iter(|| {\n            for n in &names {\n                let mut reader = format\n                    .try_get_field_data_closure(n)\n                    .expect(\"field exists\");\n                black_box(reader(data));\n            }\n        });\n    });\n\n    // After: O(1), allocation-free reads via readers resolved once per schema.\n    group.bench_function(\"cached_refs\", |b| {\n        b.iter(|| {\n            for r in &refs {\n                black_box(format.get_data(*r, data));\n            }\n        });\n    });\n\n    group.finish();\n}\n\ncriterion_group!(benches, bench_field_reads);\ncriterion_main!(benches);\n```\n</details>\n\n## Changes\n- Cache field readers per `SchemaId`. The name-finding walk and the\nclosure boxing now happen once per schema (on first sight), and every\nlater event of that schema reuses the cached closures. Per-event\nextraction becomes an `O(n)` pass with no per-field closure allocations.\n- Drop the `to_vec`. The cached closure's borrowed slice is passed\nstraight to `interpret_field_value`, so numeric fields allocate nothing.\n- Bound the cache (MAX_CACHED_SCHEMAS = 4096) against pathological\nproducers that emit unbounded distinct schemas; beyond the cap, new\nschemas fall back to the original per-event path so the map can't grow\nwithout bound.\n- Pre-size to 128 to avoid warmup rehashes.\n\n## How are these changes tested?\n- Existing unit tests and integration tests\n\n## Are there any user-facing changes?\n\nNo\n\n### Changelog\n\n* [ ] Added a `.chloggen/*.yaml` entry\n* [x] This PR is a `chore` (indicated in title)\n* [ ] This is a documentation-only PR.\n\n---------\n\nCo-authored-by: Swapnil Ashtekar <46826200+swashtek@users.noreply.github.com>",
          "timestamp": "2026-07-13T19:25:19Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/035e910605e727284dd65a849142868778665c89"
        },
        "date": 1783973112697,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.4244166612625122,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 77.97750933399514,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 81.52182620134644,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 24.243489583333332,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 24.828125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 372610.3547678259,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 367302.83099322295,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002369,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11069539.600069815,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11036837.192221768,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 30.137365318248957,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.2092945575714111,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.21288184129264,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.55239953542392,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 43.57044270833333,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 44.33203125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 2030748.7979212217,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 2006191.064233179,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002931,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 23203757.220470745,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 23124889.216156237,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.566075452210159,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
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
          "id": "035e910605e727284dd65a849142868778665c89",
          "message": "chore: Optimize per-event field extraction in the ETW receiver (#3427)\n\n## Summary\n\nCaches per-schema field readers instead of rebuilding them every event\nand drops a redundant per-field copy.\n\n## Problem\nPer event, `extract_decoded_fields` called\n`EventFormat::try_get_field_data_closure(name)` once for every field.\nEach call does two costly things:\n- Finds the field by name by re-walking the field list from index 0.\nReading all n fields is therefore `1 + 2 + ... + n = O(n^2)` name\ncomparisons per event.\n- Heap-allocates a boxed closure (`Box<dyn FnMut>`) for the result, so\n`n` allocations per event.\n\nThe result was then `.to_vec()`'d before interpretation, adding another\n`n` copies. So, a single event with `n` fields cost `O(n^2)`\nname-finding + `~2n` allocations, repeated on every event on the decode\nthread.\n\n### Performance\n\nCaching field readers per schema (instead of rebuilding a boxed closure\nper field on every event) turns per-event field extraction from an\nO(n^2) name-walk + n allocations into an O(n) reuse. A microbenchmark\nreading 16 fixed-size fields:\n\n| | time (16 fields) |\n|---|---|\n| `closure_per_field` (before) | ~990 ns |\n| `cached_refs` (after) | ~72 ns |\n\n~14x faster. The gap compounds two effects the change removes: the\nO(n^2) re-walk to find each field by name, and a heap allocation (boxed\nclosure) per field per event.\n\n<details>\n<summary>Benchmark used to measure this (criterion, against\none_collect's public API)</summary>\n\n```rust\nuse criterion::{black_box, criterion_group, criterion_main, Criterion};\nuse one_collect::event::{EventField, EventFormat, LocationType};\n\nconst FIELD_COUNT: usize = 16;\n\nfn bench_field_reads(c: &mut Criterion) {\n    // All-fixed-size schema (the common TraceLogging shape after struct\n    // flattening), so absolute offsets are valid for the cached-reader path.\n    let mut format = EventFormat::new();\n    let mut names = Vec::with_capacity(FIELD_COUNT);\n    for i in 0..FIELD_COUNT {\n        let name = format!(\"f{i}\");\n        format.add_field(EventField::new(\n            name.clone(),\n            \"u32\".to_string(),\n            LocationType::Static,\n            i * 4,\n            4,\n        ));\n        names.push(name);\n    }\n\n    let data = vec![0xABu8; FIELD_COUNT * 4];\n    let data = data.as_slice();\n\n    // Readers resolved once (a consumer caches these per schema_id).\n    let refs: Vec<_> = names\n        .iter()\n        .map(|n| format.get_field_ref(n).expect(\"field exists\"))\n        .collect();\n\n    let mut group = c.benchmark_group(\"tdh_field_reads\");\n\n    // Before: a fresh boxed closure per field, per event (re-walks the field\n    // list each call, so reading all n fields is O(n^2) plus n allocations).\n    group.bench_function(\"closure_per_field\", |b| {\n        b.iter(|| {\n            for n in &names {\n                let mut reader = format\n                    .try_get_field_data_closure(n)\n                    .expect(\"field exists\");\n                black_box(reader(data));\n            }\n        });\n    });\n\n    // After: O(1), allocation-free reads via readers resolved once per schema.\n    group.bench_function(\"cached_refs\", |b| {\n        b.iter(|| {\n            for r in &refs {\n                black_box(format.get_data(*r, data));\n            }\n        });\n    });\n\n    group.finish();\n}\n\ncriterion_group!(benches, bench_field_reads);\ncriterion_main!(benches);\n```\n</details>\n\n## Changes\n- Cache field readers per `SchemaId`. The name-finding walk and the\nclosure boxing now happen once per schema (on first sight), and every\nlater event of that schema reuses the cached closures. Per-event\nextraction becomes an `O(n)` pass with no per-field closure allocations.\n- Drop the `to_vec`. The cached closure's borrowed slice is passed\nstraight to `interpret_field_value`, so numeric fields allocate nothing.\n- Bound the cache (MAX_CACHED_SCHEMAS = 4096) against pathological\nproducers that emit unbounded distinct schemas; beyond the cap, new\nschemas fall back to the original per-event path so the map can't grow\nwithout bound.\n- Pre-size to 128 to avoid warmup rehashes.\n\n## How are these changes tested?\n- Existing unit tests and integration tests\n\n## Are there any user-facing changes?\n\nNo\n\n### Changelog\n\n* [ ] Added a `.chloggen/*.yaml` entry\n* [x] This PR is a `chore` (indicated in title)\n* [ ] This is a documentation-only PR.\n\n---------\n\nCo-authored-by: Swapnil Ashtekar <46826200+swashtek@users.noreply.github.com>",
          "timestamp": "2026-07-13T19:25:19Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/035e910605e727284dd65a849142868778665c89"
        },
        "date": 1784021845078,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.2977874279022217,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.16343858769814,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.4068358556461,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 23.04921875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.671875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 545728.2318417021,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 538645.8390275576,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00232,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 15941509.497494591,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 15926302.126325829,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 29.59553075964448,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.2519994974136353,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.22579203878144,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.58773745173745,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 42.38463541666667,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 43.2890625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 2037677.6369296436,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 2012165.924519274,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.006948,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 23207753.562261365,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 23141130.352621872,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.533717612182466,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
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
          "id": "339e7ac67cc04ab693a08cb4692203f35bd99a0f",
          "message": "feat(recordset_kql_processor): Record signals.dropped in recordset_kql_processor (#3482)\n\n# Change Summary\n\nExtends the `signals.dropped` flow metric (#2859) to the recordset_kql\nprocessor so pipelines can observe how many records it filters out.\nTogether with the earlier transform-processor change, this gives every\nKQL-based decision node the same queryable dropped count already\nprovided by `filter_processor` and `log_sampling_processor`.\n\nVery similar to recent PR #3473.\n\n### Validation\n\nRan `configs/trafficgen-flow-metrics-demo.yaml` (with `--features\nrecordset-kql-processor`), which places the recordset_kql processor as\none of four interior decision nodes in a single `ingest_pipeline` flow\nrange: the sampler keeps ~2/3, filter drops `worker-1`, transform drops\n`worker-3`, and recordset drops `worker-2`. Each decision node's drops\nare tagged with a distinct `flow.node.decision`, and the counts\nreconcile exactly against incoming/outgoing (480 − 160 − 48 − 48 − 48 =\n176).\n\n| `flow.node.decision` | Metric | Sum | Count |\n| --- | --- | ---: | ---: |\n| _(range)_ | signals.incoming | 480 | 48 |\n| sampler | signals.dropped | 160 | 48 |\n| filter | signals.dropped | 48 | 48 |\n| transform | signals.dropped | 48 | 48 |\n| **recordset** | **signals.dropped** | **48** | 48 |\n| _(range)_ | signals.outgoing | 176 | 48 |\n\nThe `recordset` row confirms the processor records `signals.dropped`\nunder its own decision attribute, exactly like the existing\n`filter`/`transform`/`sampler` decision nodes.\n\n## What issue does this PR close?\n\n<!--We highly recommend correlation of every PR to an issue-->\n\n* Related to #2859 \n\n## How are these changes tested?\n\n* Unit tests and demo config\n\n## Are there any user-facing changes?\n\n <!-- If yes, provide further info below -->\n\nAdditional `flow.dropped` metric source\n\n### Changelog\n\n<!--\nUser-facing changes need a .chloggen/*.yaml entry. Copy the\nTEMPLATE.yaml\nin go/.chloggen/ or rust/otap-dataflow/.chloggen/ and fill in the\nfields.\nIf not required, include `chore` in the PR title.\n-->\n\n* [x] Added a `.chloggen/*.yaml` entry\n* [ ] This PR is a `chore` (indicated in title)\n* [ ] This is a documentation-only PR.",
          "timestamp": "2026-07-14T23:32:04Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/339e7ac67cc04ab693a08cb4692203f35bd99a0f"
        },
        "date": 1784074481661,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.3491778373718262,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.17650557686193,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.49090458488227,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 23.782942708333334,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 24.66015625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 546445.3593022572,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 539072.8395808819,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002281,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 15996041.586166548,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 15979746.681686677,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 29.673247122973482,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.505406379699707,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.25362576305943,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 101.6696786213321,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 42.55065104166667,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 43.9765625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 2034473.0267303602,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 2044755.3832126064,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00181,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 23239937.87880859,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 23173478.02728984,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.365632324339593,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Swapnil Ashtekar",
            "username": "swashtek",
            "email": "46826200+swashtek@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "69856c386f1368f22d98802cd273dca511d31df7",
          "message": "chore: ETW tracestats metrics polling (#3425)\n\n# Change Summary\n\n1. Expose ETW session trace stats as receiver metrics\n2. Poll `query_stats(handle)` off-thread while `ProcessTrace` is blocked\n\n## What issue does this PR close?\n\n<!--We highly recommend correlation of every PR to an issue-->\nhttps://github.com/microsoft/one-collect/issues/299\n\n## How are these changes tested?\n\n## Are there any user-facing changes?\n\n <!-- If yes, provide further info below -->\n\n### Changelog\n\n<!--\nUser-facing changes need a .chloggen/*.yaml entry. Copy the\nTEMPLATE.yaml\nin go/.chloggen/ or rust/otap-dataflow/.chloggen/ and fill in the\nfields.\nIf not required, include `chore` in the PR title.\n-->\n\n* [ ] Added a `.chloggen/*.yaml` entry\n* [x] This PR is a `chore` (indicated in title)\n* [ ] This is a documentation-only PR.",
          "timestamp": "2026-07-15T01:25:54Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/69856c386f1368f22d98802cd273dca511d31df7"
        },
        "date": 1784082263934,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.00911701750010252,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 78.12573516121819,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 85.72429884879857,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 24.347265625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 24.88671875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 374376.2919476331,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 374342.15999778145,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002432,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11171478.079773191,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11138676.943864973,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 29.84295992692728,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.4709622263908386,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.22935416314841,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.60681022463207,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 42.88697916666667,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 44.21484375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 2023781.722050957,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 2033312.969926065,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003056,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 22922627.859916005,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 22871580.861477517,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.273536439768794,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
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
          "id": "ed8deab4e1a56145bcc3b1460a1507813a03c9b6",
          "message": "chore(deps): update all patch versions (#3485)\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n|---|---|---|---|\n| [syn](https://redirect.github.com/dtolnay/syn) |\nworkspace.dependencies | patch | `2.0.118` → `2.0.119` |\n| [xxhash-rust](https://redirect.github.com/DoumanAsh/xxhash-rust) |\nworkspace.dependencies | patch | `0.8.16` → `0.8.17` |\n\n---\n\n### Release Notes\n\n<details>\n<summary>dtolnay/syn (syn)</summary>\n\n###\n[`v2.0.119`](https://redirect.github.com/dtolnay/syn/releases/tag/2.0.119)\n\n[Compare\nSource](https://redirect.github.com/dtolnay/syn/compare/2.0.118...2.0.119)\n\n- Preserve attributes on tail-call expressions in statement position\n([#&#8203;1994](https://redirect.github.com/dtolnay/syn/issues/1994))\n- Parse field-representing types builtin in type position\n([#&#8203;1996](https://redirect.github.com/dtolnay/syn/issues/1996))\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: (UTC)\n\n- Branch creation\n  - \"before 8am every weekday\"\n- Automerge\n  - At any time (no schedule defined)\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n👻 **Immortal**: This PR will be recreated if closed unmerged. Get\n[config\nhelp](https://redirect.github.com/renovatebot/renovate/discussions) if\nthat's undesired.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My4yNTkuMiIsInVwZGF0ZWRJblZlciI6IjQzLjI1OS4yIiwidGFyZ2V0QnJhbmNoIjoibWFpbiIsImxhYmVscyI6WyJkZXBlbmRlbmNpZXMiXX0=-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2026-07-15T16:50:03Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/ed8deab4e1a56145bcc3b1460a1507813a03c9b6"
        },
        "date": 1784138961073,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 3.1556038856506348,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.18560118832409,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.46671005733768,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 23.866276041666666,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 24.64453125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 548880.0097880971,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 531559.5306979395,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.007578,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 16037004.857351948,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 16021765.884297319,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 30.169724990717985,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.03427635133266449,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.22819121110847,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.62290687822536,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 42.31419270833333,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 43,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 2041383.2872333697,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 2042082.9989339497,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001855,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 23181194.987402737,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 23115502.726394366,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.351739865374846,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
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
          "id": "aa051fe5dc924e81dcc0ef075de4833a0f259b6d",
          "message": "feat(metrics): Add datapoint-level enum-attribute mechanism for metric sets (#3454)\n\n# Change Summary\n\nAdds the core plumbing for datapoint-level enum attributes on metric\nsets\n\n### Outcome\n\nInstrumentation can now declare closed-set (`enum`) attributes on the\n**existing** `metric_set` unit — no new instrument family, no per-signal\nset explosion — in two flavors:\n\n- **Registration** attributes — a value fixed once at registration,\nattached to every datapoint of the set (e.g. `signal = logs` on a\njournald receiver).\n- **Measurement** attributes — values that vary per recorded datapoint\n(e.g. `signal` × `outcome` for durable-buffer loss). Each combination is\nrecorded through a generated `with(attrs)` and exported as its own\ndatapoint with the right attributes.\n\nWorst-case cardinality of a metric set is known at compile time, and a\nset that exceeds the budget (2000) is **rejected with a hard build\nerror** at the declaration site — so cardinality blowups are caught at\ncompile time rather than in production.\n\nPlain metric sets are unaffected. **No node instrumentation is migrated\nin this PR** — that is a separate sub-issue.\n\n### Usage\n\n```rust\n#[derive(Debug, Clone, Copy, AttributeEnum)]\npub enum Signal {\n    #[attribute_value = \"log-records\"] // optional rename\n    Logs,\n    Metrics,\n    Traces,\n}\n\n#[derive(Debug, Clone, Copy, AttributeEnum)]\npub enum LossOutcome {\n    Dropped,\n    Expired,\n}\n\n#[attribute_set(name = \"durable_buffer.loss.attrs\", measurement)]\n#[derive(Debug, Clone, Copy)]\npub struct LossAttributes {\n    pub signal: Signal,\n    #[attribute_key = \"loss.outcome\"] // optional rename\n    pub outcome: LossOutcome,\n}\n\n#[metric_set(name = \"processor.durable_buffer.loss\", measurement_attributes = LossAttributes)]\n#[derive(Debug, Default, Clone)]\npub struct LossMetrics {\n    #[metric(unit = \"{items}\")]\n    pub lost_items: Counter<u64>,\n}\n\nlet mut loss = LossMetrics::register(&pipeline_ctx);\nloss.with(LossAttributes {\n    signal: Signal::Metrics,\n    outcome: LossOutcome::Expired,\n})\n.lost_items\n.add(80); // signal=metrics, loss.outcome=expired\n\n#[attribute_set(name = \"signal.attrs\")]\n#[derive(Debug, Clone, Copy)]\npub struct SignalAttributes {\n    pub signal: Signal,\n}\n\n#[metric_set(name = \"receiver.journald\", registration_attributes = SignalAttributes)]\n#[derive(Debug, Default, Clone)]\npub struct JournaldMetrics {\n    #[metric(unit = \"{records}\")]\n    pub records: Counter<u64>,\n}\n\nlet mut metrics = JournaldMetrics::register(\n    &pipeline_ctx,\n    &SignalAttributes {\n        signal: Signal::Logs,\n    },\n);\nmetrics.records.add(42); // signal=log-records\n```\n\n## What issue does this PR close?\n\n<!--We highly recommend correlation of every PR to an issue-->\n\n* Part of #3300\n* Closes #3430\n\n## How are these changes tested?\n\nUnit tests\n\n## Are there any user-facing changes?\n\nNo\n\n### Changelog\n\n<!--\nUser-facing changes need a .chloggen/*.yaml entry. Copy the\nTEMPLATE.yaml\nin go/.chloggen/ or rust/otap-dataflow/.chloggen/ and fill in the\nfields.\nIf not required, include `chore` in the PR title.\n-->\n\n* [x] Added a `.chloggen/*.yaml` entry\n* [ ] This PR is a `chore` (indicated in title)\n* [ ] This is a documentation-only PR.",
          "timestamp": "2026-07-15T23:07:25Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/aa051fe5dc924e81dcc0ef075de4833a0f259b6d"
        },
        "date": 1784167912502,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.3698410987854004,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.17754011878304,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.47161875242155,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 23.101692708333335,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.8046875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 533187.7119127637,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 540491.536288211,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.005824,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 16022059.500199249,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 16001968.800448468,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 29.643497491615978,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 2.705270528793335,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.22476285407129,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.67337809678165,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 42.82825520833333,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 44.26953125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 2047588.0864650542,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1992195.2894464727,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.006069,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 23012087.540971894,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 22931800.14941464,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.551120345920381,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
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
          "id": "fa9396b54203f1b6d85bb44839aa05cdfc060ba5",
          "message": "chore(deps): update all patch versions (#3498)\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n|---|---|---|---|\n| [bitflags](https://redirect.github.com/bitflags/bitflags) |\nworkspace.dependencies | patch | `2.13.0` → `2.13.1` |\n| [clap](https://redirect.github.com/clap-rs/clap) |\nworkspace.dependencies | patch | `4.6.1` → `4.6.2` |\n\n---\n\n### Release Notes\n\n<details>\n<summary>bitflags/bitflags (bitflags)</summary>\n\n###\n[`v2.13.1`](https://redirect.github.com/bitflags/bitflags/blob/HEAD/CHANGELOG.md#2131)\n\n[Compare\nSource](https://redirect.github.com/bitflags/bitflags/compare/2.13.0...2.13.1)\n\n#### What's Changed\n\n- Lower the LLVM IR output of the generated output by\n[@&#8203;bolshoytoster](https://redirect.github.com/bolshoytoster) in\n[#&#8203;492](https://redirect.github.com/bitflags/bitflags/pull/492)\n\n#### New Contributors\n\n- [@&#8203;bolshoytoster](https://redirect.github.com/bolshoytoster)\nmade their first contribution in\n[#&#8203;492](https://redirect.github.com/bitflags/bitflags/pull/492)\n\n**Full Changelog**:\n<https://github.com/bitflags/bitflags/compare/2.13.0...2.13.1>\n\n</details>\n\n<details>\n<summary>clap-rs/clap (clap)</summary>\n\n###\n[`v4.6.2`](https://redirect.github.com/clap-rs/clap/compare/clap_complete-v4.6.1...clap_complete-v4.6.2)\n\n[Compare\nSource](https://redirect.github.com/clap-rs/clap/compare/v4.6.1...v4.6.2)\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: (UTC)\n\n- Branch creation\n  - \"before 8am every weekday\"\n- Automerge\n  - At any time (no schedule defined)\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n👻 **Immortal**: This PR will be recreated if closed unmerged. Get\n[config\nhelp](https://redirect.github.com/renovatebot/renovate/discussions) if\nthat's undesired.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My4yNTkuMiIsInVwZGF0ZWRJblZlciI6IjQzLjI1OS4yIiwidGFyZ2V0QnJhbmNoIjoibWFpbiIsImxhYmVscyI6WyJkZXBlbmRlbmNpZXMiXX0=-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2026-07-16T15:36:38Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/fa9396b54203f1b6d85bb44839aa05cdfc060ba5"
        },
        "date": 1784225879063,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.13470609486103058,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.23841387196507,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.542946766592,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 42.659244791666666,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 43.75390625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 2058694.3980010655,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 2055921.211230653,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003171,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 23363208.00992485,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 23295419.401709117,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.36386349938959,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.09926427155733109,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 87.33302665553074,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.48261991194872,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 24.0640625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 25.25390625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 438408.3340184452,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 437973.1511573969,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002363,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 13031831.825053006,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13006312.637853993,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 29.754864631804068,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
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
          "id": "a1904eee5d1e84b07820ba4a93e0a2b22c05282f",
          "message": "  feat(pdata): add retained memory sizing (#3443)\n\n# Change Summary\n\nAdds a pdata-level retained memory size API without changing existing\nencoded-size semantics.\n\nThe new API gives retention sites a way to estimate how much memory a\npayload keeps alive:\n  - `OtapArrowRecords::retained_memory_bytes()`\n  - `OtapPayload::retained_memory_bytes()`\n  - `OtapPayloadHelpers::retained_memory_bytes()`\n\nFor OTAP Arrow records, this walks Arrow buffers and dedupes shared\nbuffers within one pdata accounting call. `num_bytes()` is unchanged and\nstill represents encoded/wire size.\n\n## What issue does this PR close?\n\n* Closes #3442\n\n## How are these changes tested?\n\n  - `cargo fmt --all`\n  - `cargo check -p otap-df-pdata`\n  - `cargo clippy -p otap-df-pdata --all-targets -- -D warnings`\n  - `cargo test -p otap-df-pdata`\n  - `python3 tools/sanitycheck.py`\n\n## Are there any user-facing changes?\n\n  Yes. This adds a public pdata helper API.\n\n### Changelog\n\n* [x] Added a `.chloggen/*.yaml` entry\n* [ ] This PR is a `chore` (indicated in title)\n* [ ] This is a documentation-only PR.",
          "timestamp": "2026-07-16T19:22:55Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/a1904eee5d1e84b07820ba4a93e0a2b22c05282f"
        },
        "date": 1784254427169,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.3089894652366638,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.19209088328391,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.57960838944354,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 41.69869791666667,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 42.79296875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 2046311.3370628706,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 2052634.2235530822,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002975,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 23327336.779841732,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 23266937.801712673,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.364585327561393,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.4835917949676514,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.18086479323209,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.45552183338495,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 23.831901041666665,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 24.609375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 548080.0601926264,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 539948.7896066065,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00735,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 15998505.30274427,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 15985141.57153678,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 29.629671573854978,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          }
        ]
      }
    ]
  }
}