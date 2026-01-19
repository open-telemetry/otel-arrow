window.BENCHMARK_DATA = {
  "lastUpdate": 1768845685765,
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
          "distinct": true,
          "id": "94af57b4abe8ecb93838572f259645cc6ea9b5a7",
          "message": "Scale and Saturation test update (#1788)\n\nLocal run output is shown below. The same is uploaded to usual charts,\nso we can see how linearly we scale with CPU cores.\n\nThe saturation-tests will be refactored in future, to focus just on the\nscaling aspects (and probably renamed as scaling-tests).\n\n\n```txt\n==============================================\nAnalyzing Scaling Efficiency\n==============================================\n\nFound: 1 core(s) -> 181,463 logs/sec\nFound: 2 core(s) -> 257,679 logs/sec\nFound: 4 core(s) -> 454,159 logs/sec\n\n================================================================================\nSATURATION/SCALING TEST RESULTS - SCALING ANALYSIS\n================================================================================\n\nGoal: Verify shared-nothing architecture with linear CPU scaling\nBaseline (1 core): 181,463 logs/sec\n\n--------------------------------------------------------------------------------\nCores    Throughput (logs/sec)     Expected (linear)    Scaling Efficiency\n--------------------------------------------------------------------------------\n1        181,463                   181,463              100.00% ✅\n2        257,679                   362,927              71.00% 🟠\n4        454,159                   725,853              62.57% 🔴\n--------------------------------------------------------------------------------\n\nSUMMARY:\n  • Average Scaling Efficiency: 77.86%\n  • Minimum Scaling Efficiency: 62.57%\n  • Maximum Throughput (4 cores): 454,159 logs/sec\n  • Speedup (4 cores vs 1 core): 2.5x\n\n🟠 ACCEPTABLE: The engine shows reasonable scaling.\n   Some contention or overhead present.\n\n================================================================================\n```\n\n---------\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2026-01-15T23:41:59Z",
          "tree_id": "a78052b398334f0e19ef200ab045cbddc90a5bfd",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/94af57b4abe8ecb93838572f259645cc6ea9b5a7"
        },
        "date": 1768522382952,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "scaling_efficiency_2_cores",
            "value": 0.8715,
            "unit": "",
            "extra": "Scaling efficiency at 2 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "scaling_efficiency_4_cores",
            "value": 0.9015,
            "unit": "",
            "extra": "Scaling efficiency at 4 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "scaling_efficiency_8_cores",
            "value": 0.8015,
            "unit": "",
            "extra": "Scaling efficiency at 8 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "scaling_efficiency_16_cores",
            "value": 0.5991,
            "unit": "",
            "extra": "Scaling efficiency at 16 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "scaling_efficiency_avg",
            "value": 0.7934,
            "unit": "",
            "extra": "Average scaling efficiency across all multi-core tests (1.0 = perfect)"
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
          "distinct": false,
          "id": "632c9be1165f38afaad8f2de9a016fd1a3febdbb",
          "message": "Internal telemetry pipeline nodes config (#1794)\n\nPart of #1771.\n\nPart of #1736.\n\nFollows #1741.\n\nThis moves the HashMap<_, _> of nodes into a struct `PipelineNodes` and\nre-uses it to parse an identical graph of `internal` nodes. This\ninternal graph will be use when an internal logging provider is\nconfigured to output to an internal pipeline.",
          "timestamp": "2026-01-16T04:39:52Z",
          "tree_id": "203e6dcb1846509d12fa435a4746d4adb231ade2",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/632c9be1165f38afaad8f2de9a016fd1a3febdbb"
        },
        "date": 1768540437798,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "scaling_efficiency_2_cores",
            "value": 0.9247,
            "unit": "",
            "extra": "Scaling efficiency at 2 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "scaling_efficiency_4_cores",
            "value": 0.8458,
            "unit": "",
            "extra": "Scaling efficiency at 4 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "scaling_efficiency_8_cores",
            "value": 0.8111,
            "unit": "",
            "extra": "Scaling efficiency at 8 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "scaling_efficiency_16_cores",
            "value": 0.6763,
            "unit": "",
            "extra": "Scaling efficiency at 16 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "scaling_efficiency_avg",
            "value": 0.8145,
            "unit": "",
            "extra": "Average scaling efficiency across all multi-core tests (1.0 = perfect)"
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
          "id": "5e2ee278e5fcb023871c2215e249eb64f355f698",
          "message": "Internal logging `raw_error!` macro support (#1796)\n\nPart of https://github.com/open-telemetry/otel-arrow/pull/1771.\n\nPart of https://github.com/open-telemetry/otel-arrow/issues/1736.\n\nFollows https://github.com/open-telemetry/otel-arrow/pull/1741.\n\nThis `raw_error!` macro is different from the others in\n`internal_events.rs` in two ways:\n\n1. Supports the complete Tokio `tracing` syntax, including display and\ndebug formatters\n2. Bypasses the Tokio global dispatch and subscriber, calling into the\nraw logging layer\n\nThe use of `tracing`'s `valueset!` macro is key to supporting the whole\nsyntax for the other `otel_XXX!` macros.\n\nTest log statement prints:\n\n```\n2026-01-15T20:59:42.100Z  ERROR  otap_df_telemetry::internal_events::tests::raw error message (crates/telemetry/src/internal_events.rs:171):  [error=ConfigurationError(\"bad config\")]\n```",
          "timestamp": "2026-01-16T04:41:32Z",
          "tree_id": "96fde496f6cc09e291162cf95ead15539941d0d6",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/5e2ee278e5fcb023871c2215e249eb64f355f698"
        },
        "date": 1768541690554,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "scaling_efficiency_2_cores",
            "value": 0.9492,
            "unit": "",
            "extra": "Scaling efficiency at 2 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "scaling_efficiency_4_cores",
            "value": 0.8046,
            "unit": "",
            "extra": "Scaling efficiency at 4 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "scaling_efficiency_8_cores",
            "value": 0.8089,
            "unit": "",
            "extra": "Scaling efficiency at 8 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "scaling_efficiency_16_cores",
            "value": 0.6212,
            "unit": "",
            "extra": "Scaling efficiency at 16 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "scaling_efficiency_avg",
            "value": 0.796,
            "unit": "",
            "extra": "Average scaling efficiency across all multi-core tests (1.0 = perfect)"
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
          "id": "8e6891bb6def12af916041036a75eed2327c639a",
          "message": "Add service::telemetry::logs::providers settings for internal logging setup (#1795)\n\nPart of https://github.com/open-telemetry/otel-arrow/pull/1771.\n\nPart of https://github.com/open-telemetry/otel-arrow/issues/1736.\n\nAs documented in https://github.com/open-telemetry/otel-arrow/pull/1741.\n\n~Updates that document to match this change reflecting the prototype in\n#1771.~\n\nRevised relative to #1771.\n\nAdds LoggingProviders (choice of default logging provider for global,\nengine, and internal-telemetry threads).\nAdds ProviderMode with names to select instrumentation behavior, with\n`its` referring to internal telemetry system.\n\nNote: These settings are somehow not ideally placed. They belong also in\nthe top-level settings, or with observed_state settings. However, since\nlogging is configured with resource and level, which are part of the\nservice::telemetry config area presently, we use that structure. After\nthe bulk of #1736 is finished we can restructure.",
          "timestamp": "2026-01-16T05:28:35Z",
          "tree_id": "96d19e1d2d7270601ccadb5cccae9099af9bd16d",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/8e6891bb6def12af916041036a75eed2327c639a"
        },
        "date": 1768543368254,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "scaling_efficiency_2_cores",
            "value": 0.9387,
            "unit": "",
            "extra": "Scaling efficiency at 2 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "scaling_efficiency_4_cores",
            "value": 0.8769,
            "unit": "",
            "extra": "Scaling efficiency at 4 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "scaling_efficiency_8_cores",
            "value": 0.776,
            "unit": "",
            "extra": "Scaling efficiency at 8 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "scaling_efficiency_16_cores",
            "value": 0.6818,
            "unit": "",
            "extra": "Scaling efficiency at 16 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "scaling_efficiency_avg",
            "value": 0.8184,
            "unit": "",
            "extra": "Average scaling efficiency across all multi-core tests (1.0 = perfect)"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "a.lockett@f5.com",
            "name": "albertlockett",
            "username": "albertlockett"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "145a1d29d3f03a0396fe5c03ffff08ca27e2e20a",
          "message": "OPL parser support remove map keys operator call (#1804)\n\nCloses #1763\nCloses #1667 \n\nAdds to OPL parser support for an operator to remove keys from maps\n(attributes). The name of this operator, like in KQL, is `project-away`,\nbut there is an alias called `exclude`.\n\n```kql\nlogs | project-away attributes[\"x\"], attributes[\"y\"]\nlogs | exclude resource.attributes[\"z\"]\n// etc.\n```\n\nThis PR also uses the OPL parser in tests in the columnar query engine\nwhich use this operator. Finally, this cleans up the test code in\n`pipeline::conditional` to remove the `ConditionalTest` helper type that\nwas needed to setup the tests until we had this parser support.",
          "timestamp": "2026-01-16T16:58:24Z",
          "tree_id": "d1f134f53c1971d3c8c0c7061b1564f9f151216b",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/145a1d29d3f03a0396fe5c03ffff08ca27e2e20a"
        },
        "date": 1768584822955,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "scaling_efficiency_2_cores",
            "value": 0.9483,
            "unit": "",
            "extra": "Scaling efficiency at 2 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "scaling_efficiency_4_cores",
            "value": 0.8057,
            "unit": "",
            "extra": "Scaling efficiency at 4 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "scaling_efficiency_8_cores",
            "value": 0.8707,
            "unit": "",
            "extra": "Scaling efficiency at 8 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "scaling_efficiency_16_cores",
            "value": 0.6293,
            "unit": "",
            "extra": "Scaling efficiency at 16 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "scaling_efficiency_avg",
            "value": 0.8135,
            "unit": "",
            "extra": "Average scaling efficiency across all multi-core tests (1.0 = perfect)"
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
          "id": "0b9ab4ef7169fe98059e3626a09b4cdc76446c22",
          "message": "[otap-df-quiver] Migrate Quiver from blocking to async I/O using Tokio (#1797)\n\nMigrates Quiver from blocking to async I/O using Tokio\n\nKey Changes\n- All hot-path methods are now async: `open`, `ingest`, `next_bundle`,\n`flush`, `maintain`, `shutdown`\n- `next_bundle(id, timeout, cancel)` supports timeout and cancellation\nfor graceful shutdown\n- `poll_next_bundle(id)` provides sync non-blocking polling (renamed\nfrom old `next_bundle`)\n- Re-exports `CancellationToken` from `tokio_util` for shutdown\nsignaling\n- WAL reader remains sync (intentional - only used during crash\nrecovery)\n\n---------\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2026-01-16T17:06:43Z",
          "tree_id": "9b7a8b1e74087c163c427be2dc59bd845f7ece47",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/0b9ab4ef7169fe98059e3626a09b4cdc76446c22"
        },
        "date": 1768586084534,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "scaling_efficiency_2_cores",
            "value": 0.9008,
            "unit": "",
            "extra": "Scaling efficiency at 2 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "scaling_efficiency_4_cores",
            "value": 0.6881,
            "unit": "",
            "extra": "Scaling efficiency at 4 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "scaling_efficiency_8_cores",
            "value": 0.8,
            "unit": "",
            "extra": "Scaling efficiency at 8 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "scaling_efficiency_16_cores",
            "value": 0.6365,
            "unit": "",
            "extra": "Scaling efficiency at 16 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "scaling_efficiency_avg",
            "value": 0.7564,
            "unit": "",
            "extra": "Average scaling efficiency across all multi-core tests (1.0 = perfect)"
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
          "id": "47264a5b919925999b35dd5965cdb7075c1a814d",
          "message": "Shutdown improvements to pipeline (#1803)\n\n1. Fixes https://github.com/open-telemetry/otel-arrow/issues/1801\n2. Perf tests modified to leverage this, instead of optimistic sleep and\nwait. Most importantly, we now invoke shutdown on load-gen, engine,\nbackend (in that order), to ensure clean shutdown all throughout. No\nmore data loss!\n3. For saturation/scale test - use batch-size of 512 (OTel SDK's default\nbatch size). We previously used 200 due to data loss, but now that\ndataloss is root caused and fixes, switching to 512 as default batch\nsize.\n4. Added 24 core scenario too - this is the max we can do in the perf\nmachine, as we ran out of cores in it. (24 cores for engine means 24\nmore cores for backend and 72 for loadgen!)",
          "timestamp": "2026-01-16T18:12:31Z",
          "tree_id": "42c000a4264d3d34cac702a6cd1b0a5c1b48e1ef",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/47264a5b919925999b35dd5965cdb7075c1a814d"
        },
        "date": 1768591163641,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "scaling_efficiency_2_cores",
            "value": 0.9962,
            "unit": "",
            "extra": "Scaling efficiency at 2 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "scaling_efficiency_4_cores",
            "value": 0.805,
            "unit": "",
            "extra": "Scaling efficiency at 4 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "scaling_efficiency_8_cores",
            "value": 0.7748,
            "unit": "",
            "extra": "Scaling efficiency at 8 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "scaling_efficiency_16_cores",
            "value": 0.6041,
            "unit": "",
            "extra": "Scaling efficiency at 16 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "scaling_efficiency_24_cores",
            "value": 0.6984,
            "unit": "",
            "extra": "Scaling efficiency at 24 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "scaling_efficiency_avg",
            "value": 0.7757,
            "unit": "",
            "extra": "Average scaling efficiency across all multi-core tests (1.0 = perfect)"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "a.lockett@f5.com",
            "name": "albertlockett",
            "username": "albertlockett"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "bd628529ff4b904a539f4429d980f66a9a5adc44",
          "message": "[chore]: add CI job that checks format pest formatting (#1806)\n\nCloses #1805 \n\nAdds a CI job that fails if opl.pest grammar has not been formatted with\npestfmt.\n\nI didn't enable this for kql.pest, but happy to do so if desired",
          "timestamp": "2026-01-16T18:23:00Z",
          "tree_id": "18cd3bc766e4c9314db3ec64c2ed050a2640d8bb",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/bd628529ff4b904a539f4429d980f66a9a5adc44"
        },
        "date": 1768592614583,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "scaling_efficiency_2_cores",
            "value": 0.8708,
            "unit": "",
            "extra": "Scaling efficiency at 2 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "scaling_efficiency_4_cores",
            "value": 0.6874,
            "unit": "",
            "extra": "Scaling efficiency at 4 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "scaling_efficiency_8_cores",
            "value": 0.7761,
            "unit": "",
            "extra": "Scaling efficiency at 8 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "scaling_efficiency_16_cores",
            "value": 0.6248,
            "unit": "",
            "extra": "Scaling efficiency at 16 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "scaling_efficiency_24_cores",
            "value": 0.6433,
            "unit": "",
            "extra": "Scaling efficiency at 24 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "scaling_efficiency_avg",
            "value": 0.7205,
            "unit": "",
            "extra": "Average scaling efficiency across all multi-core tests (1.0 = perfect)"
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
          "id": "b793a1e733d3f1c1a626430c0d93e00e9e6d98e2",
          "message": "PerfTest - add passthrough scenario (#1810)\n\nAll existing tests add a dummy processor in the middle to force\nconversion into internal format. But there are real scenarios where\nengine can act as \"pass-through\" without having to do this conversion.\nThis adds a perf-test to continuously measure the throughput when\noperating as pass-through.\nModelled after saturation tests - where we put as much load as required\nto hit 100% CPU in engine. Local run shows it can do twice (minimum) the\nthroughput!",
          "timestamp": "2026-01-16T22:14:05Z",
          "tree_id": "1cf5cc0d17331750aa5a89bae24befe3b9d85c4a",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/b793a1e733d3f1c1a626430c0d93e00e9e6d98e2"
        },
        "date": 1768603837236,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "scaling_efficiency_2_cores",
            "value": 0.7595,
            "unit": "",
            "extra": "Scaling efficiency at 2 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "scaling_efficiency_4_cores",
            "value": 0.8574,
            "unit": "",
            "extra": "Scaling efficiency at 4 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "scaling_efficiency_8_cores",
            "value": 0.789,
            "unit": "",
            "extra": "Scaling efficiency at 8 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "scaling_efficiency_16_cores",
            "value": 0.6176,
            "unit": "",
            "extra": "Scaling efficiency at 16 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "scaling_efficiency_24_cores",
            "value": 0.6834,
            "unit": "",
            "extra": "Scaling efficiency at 24 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "scaling_efficiency_avg",
            "value": 0.7414,
            "unit": "",
            "extra": "Average scaling efficiency across all multi-core tests (1.0 = perfect)"
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
        "date": 1768609056354,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "scaling_efficiency_2_cores",
            "value": 0.8767,
            "unit": "",
            "extra": "Scaling efficiency at 2 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "scaling_efficiency_4_cores",
            "value": 0.719,
            "unit": "",
            "extra": "Scaling efficiency at 4 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "scaling_efficiency_8_cores",
            "value": 0.7822,
            "unit": "",
            "extra": "Scaling efficiency at 8 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "scaling_efficiency_16_cores",
            "value": 0.6594,
            "unit": "",
            "extra": "Scaling efficiency at 16 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "scaling_efficiency_24_cores",
            "value": 0.6177,
            "unit": "",
            "extra": "Scaling efficiency at 24 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "scaling_efficiency_avg",
            "value": 0.731,
            "unit": "",
            "extra": "Average scaling efficiency across all multi-core tests (1.0 = perfect)"
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
        "date": 1768621820827,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "scaling_efficiency_2_cores",
            "value": 0.7426,
            "unit": "",
            "extra": "Scaling efficiency at 2 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "scaling_efficiency_4_cores",
            "value": 0.8139,
            "unit": "",
            "extra": "Scaling efficiency at 4 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "scaling_efficiency_8_cores",
            "value": 0.6366,
            "unit": "",
            "extra": "Scaling efficiency at 8 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "scaling_efficiency_16_cores",
            "value": 0.6452,
            "unit": "",
            "extra": "Scaling efficiency at 16 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "scaling_efficiency_24_cores",
            "value": 0.645,
            "unit": "",
            "extra": "Scaling efficiency at 24 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "scaling_efficiency_avg",
            "value": 0.6967,
            "unit": "",
            "extra": "Average scaling efficiency across all multi-core tests (1.0 = perfect)"
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
        "date": 1768724559069,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "scaling_efficiency_2_cores",
            "value": 0.7499,
            "unit": "",
            "extra": "Scaling efficiency at 2 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "scaling_efficiency_4_cores",
            "value": 0.8118,
            "unit": "",
            "extra": "Scaling efficiency at 4 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "scaling_efficiency_8_cores",
            "value": 0.7913,
            "unit": "",
            "extra": "Scaling efficiency at 8 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "scaling_efficiency_16_cores",
            "value": 0.6642,
            "unit": "",
            "extra": "Scaling efficiency at 16 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "scaling_efficiency_24_cores",
            "value": 0.6219,
            "unit": "",
            "extra": "Scaling efficiency at 24 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "scaling_efficiency_avg",
            "value": 0.7278,
            "unit": "",
            "extra": "Average scaling efficiency across all multi-core tests (1.0 = perfect)"
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
          "id": "c28577df824da63d5759a149df623c30aa108c09",
          "message": "chore(deps): update dependency kubernetes to v35 (#1820)\n\nThis PR contains the following updates:\n\n| Package | Change |\n[Age](https://docs.renovatebot.com/merge-confidence/) |\n[Confidence](https://docs.renovatebot.com/merge-confidence/) |\n|---|---|---|---|\n| [kubernetes](https://redirect.github.com/kubernetes-client/python) |\n`==34.1.0` → `==35.0.0` |\n![age](https://developer.mend.io/api/mc/badges/age/pypi/kubernetes/35.0.0?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/pypi/kubernetes/34.1.0/35.0.0?slim=true)\n|\n\n---\n\n### Release Notes\n\n<details>\n<summary>kubernetes-client/python (kubernetes)</summary>\n\n###\n[`v35.0.0`](https://redirect.github.com/kubernetes-client/python/blob/HEAD/CHANGELOG.md#v3500snapshot)\n\n[Compare\nSource](https://redirect.github.com/kubernetes-client/python/compare/v34.1.0...v35.0.0)\n\nKubernetes API Version: v1.35.0\n\n##### API Change\n\n- Added `ObservedGeneration` to CustomResourceDefinition conditions.\n([kubernetes/kubernetes#134984](https://redirect.github.com/kubernetes/kubernetes/pull/134984),\n[@&#8203;michaelasp](https://redirect.github.com/michaelasp))\n- Added `WithOrigin` within `apis/core/validation` with adjusted tests.\n([kubernetes/kubernetes#132825](https://redirect.github.com/kubernetes/kubernetes/pull/132825),\n[@&#8203;PatrickLaabs](https://redirect.github.com/PatrickLaabs))\n- Added scoring for the prioritized list feature so nodes that best\nsatisfy the highest-ranked subrequests were chosen.\n([kubernetes/kubernetes#134711](https://redirect.github.com/kubernetes/kubernetes/pull/134711),\n[@&#8203;mortent](https://redirect.github.com/mortent)) \\[SIG Node,\nScheduling and Testing]\n- Added the `--min-compatibility-version` flag to `kube-apiserver`,\n`kube-controller-manager`, and `kube-scheduler`.\n([kubernetes/kubernetes#133980](https://redirect.github.com/kubernetes/kubernetes/pull/133980),\n[@&#8203;siyuanfoundation](https://redirect.github.com/siyuanfoundation))\n\\[SIG API Machinery, Architecture, Cluster Lifecycle, Etcd, Scheduling\nand Testing]\n- Added the `StorageVersionMigration` `v1beta1` API and removed the\n`v1alpha1` API.\n\nACTION REQUIRED: The `v1alpha1` API is no longer supported. Users must\nremove any `v1alpha1` resources before upgrading.\n([kubernetes/kubernetes#134784](https://redirect.github.com/kubernetes/kubernetes/pull/134784),\n[@&#8203;michaelasp](https://redirect.github.com/michaelasp)) \\[SIG API\nMachinery, Apps, Auth, Etcd and Testing]\n- Added validation to ensure `log-flush-frequency` is a positive value,\nreturning an error instead of causing a panic.\n([kubernetes/kubernetes#133540](https://redirect.github.com/kubernetes/kubernetes/pull/133540),\n[@&#8203;BenTheElder](https://redirect.github.com/BenTheElder)) \\[SIG\nArchitecture, Instrumentation, Network and Node]\n- All containers are restarted when a source container in a restart\npolicy rule exits. This alpha feature is gated behind\n`RestartAllContainersOnContainerExit`.\n([kubernetes/kubernetes#134345](https://redirect.github.com/kubernetes/kubernetes/pull/134345),\n[@&#8203;yuanwang04](https://redirect.github.com/yuanwang04)) \\[SIG\nApps, Node and Testing]\n- CSI drivers can now opt in to receive service account tokens via the\nsecrets field instead of volume context by setting\n`spec.serviceAccountTokenInSecrets: true` in the CSIDriver object. This\nprevents tokens from being exposed in logs and other outputs. The\nfeature is gated by the `CSIServiceAccountTokenSecrets` feature gate\n(beta in `v1.35`).\n([kubernetes/kubernetes#134826](https://redirect.github.com/kubernetes/kubernetes/pull/134826),\n[@&#8203;aramase](https://redirect.github.com/aramase)) \\[SIG API\nMachinery, Auth, Storage and Testing]\n- Changed kuberc configuration schema. Two new optional fields added to\nkuberc configuration, `credPluginPolicy` and `credPluginAllowlist`. This\nis documented in\n[KEP-3104](https://redirect.github.com/kubernetes/enhancements/blob/master/keps/sig-cli/3104-introduce-kuberc/README.md#allowlist-design-details)\nand documentation is added to the website by\n[kubernetes/website#52877](https://redirect.github.com/kubernetes/website/pull/52877)\n([kubernetes/kubernetes#134870](https://redirect.github.com/kubernetes/kubernetes/pull/134870),\n[@&#8203;pmengelbert](https://redirect.github.com/pmengelbert)) \\[SIG\nAPI Machinery, Architecture, Auth, CLI, Instrumentation and Testing]\n- DRA device taints: `DeviceTaintRule` status provides information about\nthe rule, including whether Pods still need to be evicted\n(`EvictionInProgress` condition). The newly added `None` effect can be\nused to preview what a `DeviceTaintRule` would do if it used the\n`NoExecute` effect and to taint devices (`device health`) without\nimmediately affecting scheduling or running Pods.\n([kubernetes/kubernetes#134152](https://redirect.github.com/kubernetes/kubernetes/pull/134152),\n[@&#8203;pohly](https://redirect.github.com/pohly)) \\[SIG API Machinery,\nApps, Auth, Node, Release, Scheduling and Testing]\n- DRA: The `DynamicResourceAllocation` feature gate for the core\nfunctionality (GA in `v1.34`) has now been locked to enabled-by-default\nand cannot be disabled anymore.\n([kubernetes/kubernetes#134452](https://redirect.github.com/kubernetes/kubernetes/pull/134452),\n[@&#8203;pohly](https://redirect.github.com/pohly)) \\[SIG Auth, Node,\nScheduling and Testing]\n- Enabled `kubectl get -o kyaml` by default. To disable it, set\n`KUBECTL_KYAML=false`.\n([kubernetes/kubernetes#133327](https://redirect.github.com/kubernetes/kubernetes/pull/133327),\n[@&#8203;thockin](https://redirect.github.com/thockin))\n- Enabled in-place resizing of pod-level resources.\n- Added `Resources` in `PodStatus` to capture resources set in the\npod-level cgroup.\n- Added `AllocatedResources` in `PodStatus` to capture resources\nrequested in the `PodSpec`.\n([kubernetes/kubernetes#132919](https://redirect.github.com/kubernetes/kubernetes/pull/132919),\n[@&#8203;ndixita](https://redirect.github.com/ndixita)) \\[SIG API\nMachinery, Apps, Architecture, Auth, CLI, Instrumentation, Node,\nScheduling and Testing]\n- Enabled the `NominatedNodeNameForExpectation` feature in\nkube-scheduler by default.\n- Enabled the `ClearingNominatedNodeNameAfterBinding` feature in\nkube-apiserver by default.\n([kubernetes/kubernetes#135103](https://redirect.github.com/kubernetes/kubernetes/pull/135103),\n[@&#8203;ania-borowiec](https://redirect.github.com/ania-borowiec))\n\\[SIG API Machinery, Apps, Architecture, Auth, Autoscaling, CLI, Cloud\nProvider, Cluster Lifecycle, Etcd, Instrumentation, Network, Node,\nScheduling, Storage and Testing]\n- Enhanced discovery responses to merge API groups and resources from\nall peer apiservers when the `UnknownVersionInteroperabilityProxy`\nfeature is enabled.\n([kubernetes/kubernetes#133648](https://redirect.github.com/kubernetes/kubernetes/pull/133648),\n[@&#8203;richabanker](https://redirect.github.com/richabanker)) \\[SIG\nAPI Machinery, Auth, Cloud Provider, Node, Scheduling and Testing]\n- Extended `core/v1` `Toleration` to support numeric comparison\noperators (`Gt`,`Lt`).\n([kubernetes/kubernetes#134665](https://redirect.github.com/kubernetes/kubernetes/pull/134665),\n[@&#8203;helayoty](https://redirect.github.com/helayoty)) \\[SIG API\nMachinery, Apps, Node, Scheduling, Testing and Windows]\n- Feature gate dependencies are now explicit, and validated at startup.\nA feature can no longer be enabled if it depends on a disabled feature.\nIn particular, this means that `AllAlpha=true` will no longer work\nwithout enabling disabled-by-default beta features that are depended on\n(either with `AllBeta=true` or explicitly enumerating the disabled\ndependencies).\n([kubernetes/kubernetes#133697](https://redirect.github.com/kubernetes/kubernetes/pull/133697),\n[@&#8203;tallclair](https://redirect.github.com/tallclair)) \\[SIG API\nMachinery, Architecture, Cluster Lifecycle and Node]\n- Generated OpenAPI model packages for API types into\n`zz_generated.model_name.go` files, accessible via the\n`OpenAPIModelName()` function. This allows API authors to declare\ndesired OpenAPI model packages instead of relying on the Go package path\nof API types.\n([kubernetes/kubernetes#131755](https://redirect.github.com/kubernetes/kubernetes/pull/131755),\n[@&#8203;jpbetz](https://redirect.github.com/jpbetz)) \\[SIG API\nMachinery, Apps, Architecture, Auth, CLI, Cloud Provider, Cluster\nLifecycle, Instrumentation, Network, Node, Scheduling, Storage and\nTesting]\n- Implemented constrained impersonation as described in\n[KEP-5284](https://kep.k8s.io/5284).\n([kubernetes/kubernetes#134803](https://redirect.github.com/kubernetes/kubernetes/pull/134803),\n[@&#8203;enj](https://redirect.github.com/enj)) \\[SIG API Machinery,\nAuth and Testing]\n- Introduced a new declarative validation tag `+k8s:customUnique` to\ncontrol listmap uniqueness.\n([kubernetes/kubernetes#134279](https://redirect.github.com/kubernetes/kubernetes/pull/134279),\n[@&#8203;yongruilin](https://redirect.github.com/yongruilin)) \\[SIG API\nMachinery and Auth]\n- Introduced a structured and versioned `v1alpha1` response for the\n`statusz` endpoint.\n([kubernetes/kubernetes#134313](https://redirect.github.com/kubernetes/kubernetes/pull/134313),\n[@&#8203;richabanker](https://redirect.github.com/richabanker)) \\[SIG\nAPI Machinery, Architecture, Instrumentation, Network, Node, Scheduling\nand Testing]\n- Introduced a structured and versioned `v1alpha1` response format for\nthe `flagz` endpoint.\n([kubernetes/kubernetes#134995](https://redirect.github.com/kubernetes/kubernetes/pull/134995),\n[@&#8203;yongruilin](https://redirect.github.com/yongruilin)) \\[SIG API\nMachinery, Architecture, Instrumentation, Network, Node, Scheduling and\nTesting]\n- Introduced the GangScheduling kube-scheduler plugin to support\n\"all-or-nothing\" scheduling using the `scheduling.k8s.io/v1alpha1`\nWorkload API.\n([kubernetes/kubernetes#134722](https://redirect.github.com/kubernetes/kubernetes/pull/134722),\n[@&#8203;macsko](https://redirect.github.com/macsko)) \\[SIG API\nMachinery, Apps, Auth, CLI, Etcd, Scheduling and Testing]\n- Introduced the Node Declared Features capability (alpha), which\nincludes:\n- A new `Node.Status.DeclaredFeatures` field for publishing\nnode-specific features.\n- A `component-helpers` library for feature registration and inference.\n- A `NodeDeclaredFeatures` scheduler plugin to match pods with nodes\nthat provide required features.\n- A `NodeDeclaredFeatureValidator` admission plugin to validate pod\nupdates against a node's declared features.\n([kubernetes/kubernetes#133389](https://redirect.github.com/kubernetes/kubernetes/pull/133389),\n[@&#8203;pravk03](https://redirect.github.com/pravk03)) \\[SIG API\nMachinery, Apps, Node, Release, Scheduling and Testing]\n- Introduced the `scheduling.k8s.io/v1alpha1` Workload API to express\nworkload-level scheduling requirements and allow the kube-scheduler to\nact on them.\n([kubernetes/kubernetes#134564](https://redirect.github.com/kubernetes/kubernetes/pull/134564),\n[@&#8203;macsko](https://redirect.github.com/macsko)) \\[SIG API\nMachinery, Apps, CLI, Etcd, Scheduling and Testing]\n- Introduced the alpha `MutableSchedulingDirectivesForSuspendedJobs`\nfeature gate (disabled by default), which allows mutating a Job's\nscheduling directives while the Job is suspended.\nIt also updates the Job controller to clears the `status.startTime`\nfield for suspended Jobs.\n([kubernetes/kubernetes#135104](https://redirect.github.com/kubernetes/kubernetes/pull/135104),\n[@&#8203;mimowo](https://redirect.github.com/mimowo)) \\[SIG Apps and\nTesting]\n- Kube-apiserver: Fixed a `v1.34` regression in\n`CustomResourceDefinition` handling that incorrectly warned about\nunrecognized formats on number and integer properties.\n([kubernetes/kubernetes#133896](https://redirect.github.com/kubernetes/kubernetes/pull/133896),\n[@&#8203;yongruilin](https://redirect.github.com/yongruilin)) \\[SIG API\nMachinery, Apps, Architecture, Auth, CLI, Cloud Provider, Contributor\nExperience, Network, Node and Scheduling]\n- Kube-apiserver: Fixed a possible panic validating a custom resource\nwhose `CustomResourceDefinition` indicates a status subresource exists,\nbut which does not define a `status` property in the `openAPIV3Schema`.\n([kubernetes/kubernetes#133721](https://redirect.github.com/kubernetes/kubernetes/pull/133721),\n[@&#8203;fusida](https://redirect.github.com/fusida)) \\[SIG API\nMachinery, Apps, Architecture, Auth, Autoscaling, CLI, Cloud Provider,\nCluster Lifecycle, Etcd, Instrumentation, Network, Node, Release,\nScheduling, Storage and Testing]\n- Kubernetes API Go types removed runtime use of the\n`github.com/gogo/protobuf` library, and are no longer registered into\nthe global gogo type registry. Kubernetes API Go types were not suitable\nfor use with the `google.golang.org/protobuf` library, and no longer\nimplement `ProtoMessage()` by default to avoid accidental incompatible\nuse. If removal of these marker methods impacts your use, it can be\nre-enabled for one more release with a\n`kubernetes_protomessage_one_more_release` build tag, but will be\nremoved in `v1.36`.\n([kubernetes/kubernetes#134256](https://redirect.github.com/kubernetes/kubernetes/pull/134256),\n[@&#8203;liggitt](https://redirect.github.com/liggitt)) \\[SIG API\nMachinery, Apps, Architecture, Auth, CLI, Cluster Lifecycle,\nInstrumentation, Network, Node, Scheduling and Storage]\n- Made node affinity in Persistent Volume mutable.\n([kubernetes/kubernetes#134339](https://redirect.github.com/kubernetes/kubernetes/pull/134339),\n[@&#8203;huww98](https://redirect.github.com/huww98)) \\[SIG API\nMachinery, Apps and Node]\n- Moved the `ImagePullIntent` and `ImagePulledRecord` objects used by\nthe kubelet to track image pulls to the `v1beta1` API version.\n([kubernetes/kubernetes#132579](https://redirect.github.com/kubernetes/kubernetes/pull/132579),\n[@&#8203;stlaz](https://redirect.github.com/stlaz)) \\[SIG Auth and Node]\n- Pod resize now only allows CPU and memory resources; other resource\ntypes are forbidden.\n([kubernetes/kubernetes#135084](https://redirect.github.com/kubernetes/kubernetes/pull/135084),\n[@&#8203;tallclair](https://redirect.github.com/tallclair)) \\[SIG Apps,\nNode and Testing]\n- Prevented Pods from being scheduled onto nodes that lack the required\nCSI driver.\n([kubernetes/kubernetes#135012](https://redirect.github.com/kubernetes/kubernetes/pull/135012),\n[@&#8203;gnufied](https://redirect.github.com/gnufied)) \\[SIG API\nMachinery, Scheduling, Storage and Testing]\n- Promoted HPA configurable tolerance to beta. The\n`HPAConfigurableTolerance` feature gate has now been enabled by default.\n([kubernetes/kubernetes#133128](https://redirect.github.com/kubernetes/kubernetes/pull/133128),\n[@&#8203;jm-franc](https://redirect.github.com/jm-franc)) \\[SIG API\nMachinery and Autoscaling]\n- Promoted ReplicaSet and Deployment `.status.terminatingReplicas`\ntracking to beta. The `DeploymentReplicaSetTerminatingReplicas` feature\ngate is now enabled by default.\n([kubernetes/kubernetes#133087](https://redirect.github.com/kubernetes/kubernetes/pull/133087),\n[@&#8203;atiratree](https://redirect.github.com/atiratree)) \\[SIG API\nMachinery, Apps and Testing]\n- Promoted `PodObservedGenerationTracking` to GA.\n([kubernetes/kubernetes#134948](https://redirect.github.com/kubernetes/kubernetes/pull/134948),\n[@&#8203;natasha41575](https://redirect.github.com/natasha41575)) \\[SIG\nAPI Machinery, Apps, Node, Scheduling and Testing]\n- Promoted the `JobManagedBy` feature to general availability. The\n`JobManagedBy` feature gate was locked to `true` and will be removed in\na future Kubernetes release.\n([kubernetes/kubernetes#135080](https://redirect.github.com/kubernetes/kubernetes/pull/135080),\n[@&#8203;dejanzele](https://redirect.github.com/dejanzele)) \\[SIG API\nMachinery, Apps and Testing]\n- Promoted the `MaxUnavailableStatefulSet` feature to beta and enabling\nit by default.\n([kubernetes/kubernetes#133153](https://redirect.github.com/kubernetes/kubernetes/pull/133153),\n[@&#8203;helayoty](https://redirect.github.com/helayoty)) \\[SIG API\nMachinery and Apps]\n- Removed the `StrictCostEnforcementForVAP` and\n`StrictCostEnforcementForWebhooks` feature gates, which were locked\nsince `v1.32`.\n([kubernetes/kubernetes#134994](https://redirect.github.com/kubernetes/kubernetes/pull/134994),\n[@&#8203;liggitt](https://redirect.github.com/liggitt)) \\[SIG API\nMachinery, Auth, Node and Testing]\n- Scheduler: Added the `bindingTimeout` argument to the DynamicResources\nplugin configuration, allowing customization of the wait duration in\n`PreBind` for device binding conditions.\nDefaults to 10 minutes when `DRADeviceBindingConditions` and\n`DRAResourceClaimDeviceStatus` are both enabled.\n([kubernetes/kubernetes#134905](https://redirect.github.com/kubernetes/kubernetes/pull/134905),\n[@&#8203;fj-naji](https://redirect.github.com/fj-naji)) \\[SIG Node and\nScheduling]\n- The DRA device taints and toleration feature received a separate\nfeature gate, `DRADeviceTaintRules`, which controlled support for\n`DeviceTaintRules`. This allowed disabling it while keeping\n`DRADeviceTaints` enabled so that tainting via `ResourceSlices`\ncontinued to work.\n([kubernetes/kubernetes#135068](https://redirect.github.com/kubernetes/kubernetes/pull/135068),\n[@&#8203;pohly](https://redirect.github.com/pohly)) \\[SIG API Machinery,\nApps, Auth, Node, Scheduling and Testing]\n- The Pod Certificates feature moved to beta. The\n`PodCertificateRequest` feature gate is set disabled by default. To use\nthe feature, users must enable the certificates API groups in `v1beta1`\nand enable the `PodCertificateRequest` feature gate. The\n`UserAnnotations` field was added to the `PodCertificateProjection` API\nand the corresponding `UnverifiedUserAnnotations` field was added to the\n`PodCertificateRequest` API.\n([kubernetes/kubernetes#134624](https://redirect.github.com/kubernetes/kubernetes/pull/134624),\n[@&#8203;yt2985](https://redirect.github.com/yt2985)) \\[SIG API\nMachinery, Apps, Auth, Etcd, Instrumentation, Node and Testing]\n- The `KubeletEnsureSecretPulledImages` feature was promoted to Beta and\nenabled by default.\n([kubernetes/kubernetes#135228](https://redirect.github.com/kubernetes/kubernetes/pull/135228),\n[@&#8203;aramase](https://redirect.github.com/aramase)) \\[SIG Auth, Node\nand Testing]\n- The `PreferSameZone` and `PreferSameNode` values for the Service\n  `trafficDistribution` field graduated to general availability. The\n  `PreferClose` value is now deprecated in favor of the more explicit\n`PreferSameZone`.\n([kubernetes/kubernetes#134457](https://redirect.github.com/kubernetes/kubernetes/pull/134457),\n[@&#8203;danwinship](https://redirect.github.com/danwinship)) \\[SIG API\nMachinery, Apps, Network and Testing]\n- Updated `ResourceQuota` to count device class requests within a\n`ResourceClaim` as two additional quotas when the `DRAExtendedResource`\nfeature is enabled:\n- `requests.deviceclass.resource.k8s.io/<deviceclass>` is charged based\non the worst-case number of devices requested.\n- Device classes mapping to an extended resource now consume\n`requests.<extended resource name>`.\n([kubernetes/kubernetes#134210](https://redirect.github.com/kubernetes/kubernetes/pull/134210),\n[@&#8203;yliaog](https://redirect.github.com/yliaog)) \\[SIG API\nMachinery, Apps, Node, Scheduling and Testing]\n- Updated storage version for `MutatingAdmissionPolicy` to `v1beta1`.\n([kubernetes/kubernetes#133715](https://redirect.github.com/kubernetes/kubernetes/pull/133715),\n[@&#8203;cici37](https://redirect.github.com/cici37)) \\[SIG API\nMachinery, Etcd and Testing]\n- Updated the Partitionable Devices feature to support referencing\ncounter sets across ResourceSlices within the same resource pool.\nDevices from incomplete pools were no longer considered for allocation.\nThis change introduced backwards-incompatible updates to the alpha\nfeature, requiring any ResourceSlices using it to be removed before\nupgrading or downgrading between v1.34 and v1.35.\n([kubernetes/kubernetes#134189](https://redirect.github.com/kubernetes/kubernetes/pull/134189),\n[@&#8203;mortent](https://redirect.github.com/mortent)) \\[SIG API\nMachinery, Node, Scheduling and Testing]\n- Upgraded the `PodObservedGenerationTracking` feature to beta in\n`v1.34` and removed the alpha version description from the OpenAPI\nspecification.\n([kubernetes/kubernetes#133883](https://redirect.github.com/kubernetes/kubernetes/pull/133883),\n[@&#8203;yangjunmyfm192085](https://redirect.github.com/yangjunmyfm192085))\n- Add scoring for the prioritized list feature so that the node that can\nsatisfy the best ranked subrequests are chosen.\n([kubernetes/kubernetes#134711](https://redirect.github.com/kubernetes/kubernetes/pull/134711),\n[@&#8203;mortent](https://redirect.github.com/mortent)) \\[SIG Node,\nScheduling and Testing]\n- Allows restart all containers when the source container exits with a\nmatching restart policy rule. This is an alpha feature behind feature\ngate RestartAllContainersOnContainerExit.\n([kubernetes/kubernetes#134345](https://redirect.github.com/kubernetes/kubernetes/pull/134345),\n[@&#8203;yuanwang04](https://redirect.github.com/yuanwang04)) \\[SIG\nApps, Node and Testing]\n- Changed kuberc configuration schema. Two new optional fields added to\nkuberc configuration, `credPluginPolicy` and `credPluginAllowlist`. This\nis documented in\n[KEP-3104](https://redirect.github.com/kubernetes/enhancements/blob/master/keps/sig-cli/3104-introduce-kuberc/README.md#allowlist-design-details)\nand documentation is added to the website by\n[kubernetes/website#52877](https://redirect.github.com/kubernetes/website/pull/52877)\n([kubernetes/kubernetes#134870](https://redirect.github.com/kubernetes/kubernetes/pull/134870),\n[@&#8203;pmengelbert](https://redirect.github.com/pmengelbert)) \\[SIG\nAPI Machinery, Architecture, Auth, CLI, Instrumentation and Testing]\n- Enhanced discovery response to support merged API groups/resources\nfrom all peer apiservers when UnknownVersionInteroperabilityProxy\nfeature is enabled\n([kubernetes/kubernetes#133648](https://redirect.github.com/kubernetes/kubernetes/pull/133648),\n[@&#8203;richabanker](https://redirect.github.com/richabanker)) \\[SIG\nAPI Machinery, Auth, Cloud Provider, Node, Scheduling and Testing]\n- Extend `core/v1 Toleration` to support numeric comparison operators\n(`Gt`, `Lt`).\n([kubernetes/kubernetes#134665](https://redirect.github.com/kubernetes/kubernetes/pull/134665),\n[@&#8203;helayoty](https://redirect.github.com/helayoty)) \\[SIG API\nMachinery, Apps, Node, Scheduling, Testing and Windows]\n- Features: NominatedNodeNameForExpectation in kube-scheduler and\nCleaeringNominatedNodeNameAfterBinding in kube-apiserver are now enabled\nby default.\n([kubernetes/kubernetes#135103](https://redirect.github.com/kubernetes/kubernetes/pull/135103),\n[@&#8203;ania-borowiec](https://redirect.github.com/ania-borowiec))\n\\[SIG API Machinery, Apps, Architecture, Auth, Autoscaling, CLI, Cloud\nProvider, Cluster Lifecycle, Etcd, Instrumentation, Network, Node,\nScheduling, Storage and Testing]\n- Implement changes to prevent pod scheduling to a node without CSI\ndriver\n([kubernetes/kubernetes#135012](https://redirect.github.com/kubernetes/kubernetes/pull/135012),\n[@&#8203;gnufied](https://redirect.github.com/gnufied)) \\[SIG API\nMachinery, Scheduling, Storage and Testing]\n- Introduce scheduling.k8s.io/v1alpha1 Workload API to allow for\nexpressing workload-level scheduling requirements and let kube-scheduler\nact on those.\n([kubernetes/kubernetes#134564](https://redirect.github.com/kubernetes/kubernetes/pull/134564),\n[@&#8203;macsko](https://redirect.github.com/macsko)) \\[SIG API\nMachinery, Apps, CLI, Etcd, Scheduling and Testing]\n- Introduce the alpha MutableSchedulingDirectivesForSuspendedJobs\nfeature gate (disabled by default) which:\n  1. allows to mutate Job's scheduling directives for suspended Jobs\n2. makes the Job controller to clear the status.startTime field for\nsuspended Jobs\n([kubernetes/kubernetes#135104](https://redirect.github.com/kubernetes/kubernetes/pull/135104),\n[@&#8203;mimowo](https://redirect.github.com/mimowo)) \\[SIG Apps and\nTesting]\n- Introduced GangScheduling kube-scheduler plugin to enable\n\"all-or-nothing\" scheduling. Workload API in scheduling.k8s.io/v1alpha1\nis used to express the desired policy.\n([kubernetes/kubernetes#134722](https://redirect.github.com/kubernetes/kubernetes/pull/134722),\n[@&#8203;macsko](https://redirect.github.com/macsko)) \\[SIG API\nMachinery, Apps, Auth, CLI, Etcd, Scheduling and Testing]\n- PV node affinity is now mutable.\n([kubernetes/kubernetes#134339](https://redirect.github.com/kubernetes/kubernetes/pull/134339),\n[@&#8203;huww98](https://redirect.github.com/huww98)) \\[SIG API\nMachinery, Apps and Node]\n- ResourceQuota now counts device class requests within a ResourceClaim\nobject as consuming two additional quotas when the DRAExtendedResource\nfeature is enabled:\n- `requests.deviceclass.resource.k8s.io/<deviceclass>` with a quantity\nequal to the worst case count of devices requested\n- requests for device classes that map to an extended resource consume\n`requests.<extended resource name>`\n([kubernetes/kubernetes#134210](https://redirect.github.com/kubernetes/kubernetes/pull/134210),\n[@&#8203;yliaog](https://redirect.github.com/yliaog)) \\[SIG API\nMachinery, Apps, Node, Scheduling and Testing]\n- The DRA device taints and toleration feature now has a separate\nfeature gate, DRADeviceTaintRules, which controls whether support for\nDeviceTaintRules is enabled. It is possible to disable that and keep\nDRADeviceTaints enabled, in which case tainting by DRA drivers through\nResourceSlices continues to work.\n([kubernetes/kubernetes#135068](https://redirect.github.com/kubernetes/kubernetes/pull/135068),\n[@&#8203;pohly](https://redirect.github.com/pohly)) \\[SIG API Machinery,\nApps, Auth, Node, Scheduling and Testing]\n- The ImagePullIntent and ImagePulledRecord objects used by kubelet to\nstore information about image pulls have been moved to the v1beta1 API\nversion.\n([kubernetes/kubernetes#132579](https://redirect.github.com/kubernetes/kubernetes/pull/132579),\n[@&#8203;stlaz](https://redirect.github.com/stlaz)) \\[SIG Auth and Node]\n- The KubeletEnsureSecretPulledImages feature is now beta and enabled by\ndefault.\n([kubernetes/kubernetes#135228](https://redirect.github.com/kubernetes/kubernetes/pull/135228),\n[@&#8203;aramase](https://redirect.github.com/aramase)) \\[SIG Auth, Node\nand Testing]\n- This change adds a new alpha feature Node Declared Features, which\nincludes:\n- A new `Node.Status.DeclaredFeatures` field for Kubelet to publish\nnode-specific features.\n- A library in `component-helpers` for feature registration and\ninference.\n- A scheduler plugin (`NodeDeclaredFeatures`) scheduler plugin to match\npods with nodes that provide their required features.\n- An admission plugin (`NodeDeclaredFeatureValidator`) to validate pod\nupdates against a node's declared features.\n([kubernetes/kubernetes#133389](https://redirect.github.com/kubernetes/kubernetes/pull/133389),\n[@&#8203;pravk03](https://redirect.github.com/pravk03)) \\[SIG API\nMachinery, Apps, Node, Release, Scheduling and Testing]\n- This change allows In Place Resize of Pod Level Resources\n- Add Resources in PodStatus to capture resources set at pod-level\ncgroup\n- Add AllocatedResources in PodStatus to capture resources requested in\nthe PodSpec\n([kubernetes/kubernetes#132919](https://redirect.github.com/kubernetes/kubernetes/pull/132919),\n[@&#8203;ndixita](https://redirect.github.com/ndixita)) \\[SIG API\nMachinery, Apps, Architecture, Auth, CLI, Instrumentation, Node,\nScheduling and Testing]\n- Updates to the Partitionable Devices feature which allows for\nreferencing counter sets across different ResourceSlices within the same\nresource pool.\n\n  Devices from incomplete pools are no longer considered for allocation.\n\nThis contains backwards incompatible changes to the Partitionable\nDevices alpha feature, so any ResourceSlices that uses the feature\nshould be removed prior to upgrading or downgrading between 1.34 and\n1.35.\n([kubernetes/kubernetes#134189](https://redirect.github.com/kubernetes/kubernetes/pull/134189),\n[@&#8203;mortent](https://redirect.github.com/mortent)) \\[SIG API\nMachinery, Node, Scheduling and Testing]\n- Add ObservedGeneration to CustomResourceDefinition Conditions.\n([kubernetes/kubernetes#134984](https://redirect.github.com/kubernetes/kubernetes/pull/134984),\n[@&#8203;michaelasp](https://redirect.github.com/michaelasp)) \\[SIG API\nMachinery]\n- Add StorageVersionMigration v1beta1 api and remove the v1alpha API.\n\n  Any use of the v1alpha1 api is no longer supported and\nusers must remove any v1alpha1 resources prior to upgrade.\n([kubernetes/kubernetes#134784](https://redirect.github.com/kubernetes/kubernetes/pull/134784),\n[@&#8203;michaelasp](https://redirect.github.com/michaelasp)) \\[SIG API\nMachinery, Apps, Auth, Etcd and Testing]\n- CSI drivers can now opt-in to receive service account tokens via the\nsecrets field instead of volume context by setting\n`spec.serviceAccountTokenInSecrets: true` in the CSIDriver object. This\nprevents tokens from being exposed in logs and other outputs. The\nfeature is gated by the `CSIServiceAccountTokenSecrets` feature gate\n(Beta in v1.35).\n([kubernetes/kubernetes#134826](https://redirect.github.com/kubernetes/kubernetes/pull/134826),\n[@&#8203;aramase](https://redirect.github.com/aramase)) \\[SIG API\nMachinery, Auth, Storage and Testing]\n- DRA device taints: DeviceTaintRule status provided information about\nthe rule, in particular whether pods still need to be evicted\n(\"EvictionInProgress\" condition). The new \"None\" effect can be used to\npreview what a DeviceTaintRule would do if it used the \"NoExecute\"\neffect and to taint devices (\"device health\") without immediately\naffecting scheduling or running pods.\n([kubernetes/kubernetes#134152](https://redirect.github.com/kubernetes/kubernetes/pull/134152),\n[@&#8203;pohly](https://redirect.github.com/pohly)) \\[SIG API Machinery,\nApps, Auth, Node, Release, Scheduling and Testing]\n- DRA: the DynamicResourceAllocation feature gate for the core\nfunctionality (GA in 1.34) is now locked to enabled-by-default and thus\ncannot be disabled anymore.\n([kubernetes/kubernetes#134452](https://redirect.github.com/kubernetes/kubernetes/pull/134452),\n[@&#8203;pohly](https://redirect.github.com/pohly)) \\[SIG Auth, Node,\nScheduling and Testing]\n- Forbid adding resources other than CPU & memory on pod resize.\n([kubernetes/kubernetes#135084](https://redirect.github.com/kubernetes/kubernetes/pull/135084),\n[@&#8203;tallclair](https://redirect.github.com/tallclair)) \\[SIG Apps,\nNode and Testing]\n- Implement constrained impersonation as described in\n<https://kep.k8s.io/5284>\n([kubernetes/kubernetes#134803](https://redirect.github.com/kubernetes/kubernetes/pull/134803),\n[@&#8203;enj](https://redirect.github.com/enj)) \\[SIG API Machinery,\nAuth and Testing]\n- Introduces a structured and versioned v1alpha1 response for flagz\n([kubernetes/kubernetes#134995](https://redirect.github.com/kubernetes/kubernetes/pull/134995),\n[@&#8203;yongruilin](https://redirect.github.com/yongruilin)) \\[SIG API\nMachinery, Architecture, Instrumentation, Network, Node, Scheduling and\nTesting]\n- Introduces a structured and versioned v1alpha1 response for statusz\n([kubernetes/kubernetes#134313](https://redirect.github.com/kubernetes/kubernetes/pull/134313),\n[@&#8203;richabanker](https://redirect.github.com/richabanker)) \\[SIG\nAPI Machinery, Architecture, Instrumentation, Network, Node, Scheduling\nand Testing]\n- New `--min-compatibility-version` flag for apiserver, kcm and kube\nscheduler\n([kubernetes/kubernetes#133980](https://redirect.github.com/kubernetes/kubernetes/pull/133980),\n[@&#8203;siyuanfoundation](https://redirect.github.com/siyuanfoundation))\n\\[SIG API Machinery, Architecture, Cluster Lifecycle, Etcd, Scheduling\nand Testing]\n- Promote PodObservedGenerationTracking to GA.\n([kubernetes/kubernetes#134948](https://redirect.github.com/kubernetes/kubernetes/pull/134948),\n[@&#8203;natasha41575](https://redirect.github.com/natasha41575)) \\[SIG\nAPI Machinery, Apps, Node, Scheduling and Testing]\n- Promoted Job Managed By to general availability. The `JobManagedBy`\nfeature gate is now locked to true, and will be removed in a future\nrelease of Kubernetes.\n([kubernetes/kubernetes#135080](https://redirect.github.com/kubernetes/kubernetes/pull/135080),\n[@&#8203;dejanzele](https://redirect.github.com/dejanzele)) \\[SIG API\nMachinery, Apps and Testing]\n- Promoted ReplicaSet and Deployment `.status.terminatingReplicas`\ntracking to beta. The `DeploymentReplicaSetTerminatingReplicas` feature\ngate is now enabled by default.\n([kubernetes/kubernetes#133087](https://redirect.github.com/kubernetes/kubernetes/pull/133087),\n[@&#8203;atiratree](https://redirect.github.com/atiratree)) \\[SIG API\nMachinery, Apps and Testing]\n- Scheduler: added a new `bindingTimeout` argument to the\nDynamicResources plugin configuration.\nThis allows customizing the wait duration in PreBind for device binding\nconditions.\nDefaults to 10 minutes when DRADeviceBindingConditions and\nDRAResourceClaimDeviceStatus are both enabled.\n([kubernetes/kubernetes#134905](https://redirect.github.com/kubernetes/kubernetes/pull/134905),\n[@&#8203;fj-naji](https://redirect.github.com/fj-naji)) \\[SIG Node and\nScheduling]\n- The Pod Certificates feature is moving to beta. The\nPodCertificateRequest feature gate is still set false by default. To use\nthe feature, users will need to enable the certificates API groups in\nv1beta1 and enable the feature gate PodCertificateRequest. A new field\nUserAnnotations is added to the PodCertificateProjection API and the\ncorresponding UnverifiedUserAnnotations is added to the\nPodCertificateRequest API.\n([kubernetes/kubernetes#134624](https://redirect.github.com/kubernetes/kubernetes/pull/134624),\n[@&#8203;yt2985](https://redirect.github.com/yt2985)) \\[SIG API\nMachinery, Apps, Auth, Etcd, Instrumentation, Node and Testing]\n- The StrictCostEnforcementForVAP and StrictCostEnforcementForWebhooks\nfeature gates, locked on since 1.32, have been removed\n([kubernetes/kubernetes#134994](https://redirect.github.com/kubernetes/kubernetes/pull/134994),\n[@&#8203;liggitt](https://redirect.github.com/liggitt)) \\[SIG API\nMachinery, Auth, Node and Testing]\n- The `PreferSameZone` and `PreferSameNode` values for Service's\n`trafficDistribution` field are now GA. The old value `PreferClose` is\nnow\ndeprecated in favor of the more-explicit `PreferSameZone`.\n([kubernetes/kubernetes#134457](https://redirect.github.com/kubernetes/kubernetes/pull/134457),\n[@&#8203;danwinship](https://redirect.github.com/danwinship)) \\[SIG API\nMachinery, Apps, Network and Testing]\n- Kube-apiserver: fix a possible panic validating a custom resource\nwhose CustomResourceDefinition indicates a status subresource exists,\nbut which does not define a `status` property in the `openAPIV3Schema`\n([kubernetes/kubernetes#133721](https://redirect.github.com/kubernetes/kubernetes/pull/133721),\n[@&#8203;fusida](https://redirect.github.com/fusida)) \\[SIG API\nMachinery, Apps, Architecture, Auth, Autoscaling, CLI, Cloud Provider,\nCluster Lifecycle, Etcd, Instrumentation, Network, Node, Release,\nScheduling, Storage and Testing]\n- Kubernetes API Go types removed runtime use of the\ngithub.com/gogo/protobuf library, and are no longer registered into the\nglobal gogo type registry. Kubernetes API Go types were not suitable for\nuse with the google.golang.org/protobuf library, and no longer implement\n`ProtoMessage()` by default to avoid accidental incompatible use. If\nremoval of these marker methods impacts your use, it can be re-enabled\nfor one more release with a `kubernetes_protomessage_one_more_release`\nbuild tag, but will be removed in 1.36.\n([kubernetes/kubernetes#134256](https://redirect.github.com/kubernetes/kubernetes/pull/134256),\n[@&#8203;liggitt](https://redirect.github.com/liggitt)) \\[SIG API\nMachinery, Apps, Architecture, Auth, CLI, Cluster Lifecycle,\nInstrumentation, Network, Node, Scheduling and Storage]\n- Promoted HPA configurable tolerance to beta. The\n`HPAConfigurableTolerance` feature gate is now enabled by default.\n([kubernetes/kubernetes#133128](https://redirect.github.com/kubernetes/kubernetes/pull/133128),\n[@&#8203;jm-franc](https://redirect.github.com/jm-franc)) \\[SIG API\nMachinery and Autoscaling]\n- The MaxUnavailableStatefulSet feature is now beta and enabled by\ndefault.\n([kubernetes/kubernetes#133153](https://redirect.github.com/kubernetes/kubernetes/pull/133153),\n[@&#8203;helayoty](https://redirect.github.com/helayoty)) \\[SIG API\nMachinery and Apps]\n- Added WithOrigin within apis/core/validation with adjusted tests\n([kubernetes/kubernetes#132825](https://redirect.github.com/kubernetes/kubernetes/pull/132825),\n[@&#8203;PatrickLaabs](https://redirect.github.com/PatrickLaabs)) \\[SIG\nApps]\n- Component-base: validate that log-flush-frequency is positive and\nreturn an error instead of panic-ing\n([kubernetes/kubernetes#133540](https://redirect.github.com/kubernetes/kubernetes/pull/133540),\n[@&#8203;BenTheElder](https://redirect.github.com/BenTheElder)) \\[SIG\nArchitecture, Instrumentation, Network and Node]\n- Feature gate dependencies are now explicit, and validated at startup.\nA feature can no longer be enabled if it depends on a disabled feature.\nIn particular, this means that `AllAlpha=true` will no longer work\nwithout enabling disabled-by-default beta features that are depended on\n(either with `AllBeta=true` or explicitly enumerating the disabled\ndependencies).\n([kubernetes/kubernetes#133697](https://redirect.github.com/kubernetes/kubernetes/pull/133697),\n[@&#8203;tallclair](https://redirect.github.com/tallclair)) \\[SIG API\nMachinery, Architecture, Cluster Lifecycle and Node]\n- In version 1.34, the PodObservedGenerationTracking feature has been\nupgraded to beta, and the description of the alpha version in the\nopenapi has been removed.\n([kubernetes/kubernetes#133883](https://redirect.github.com/kubernetes/kubernetes/pull/133883),\n[@&#8203;yangjunmyfm192085](https://redirect.github.com/yangjunmyfm192085))\n\\[SIG Apps]\n- Introduce a new declarative validation tag +k8s:customUnique to\ncontrol listmap uniqueness\n([kubernetes/kubernetes#134279](https://redirect.github.com/kubernetes/kubernetes/pull/134279),\n[@&#8203;yongruilin](https://redirect.github.com/yongruilin)) \\[SIG API\nMachinery and Auth]\n- Kube-apiserver: Fixed a 1.34 regression in CustomResourceDefinition\nhandling that incorrectly warned about unrecognized formats on number\nand integer properties\n([kubernetes/kubernetes#133896](https://redirect.github.com/kubernetes/kubernetes/pull/133896),\n[@&#8203;yongruilin](https://redirect.github.com/yongruilin)) \\[SIG API\nMachinery, Apps, Architecture, Auth, CLI, Cloud Provider, Contributor\nExperience, Network, Node and Scheduling]\n- OpenAPI model packages of API types are generated into\n`zz_generated.model_name.go` files and are accessible using the\n`OpenAPIModelName()` function. This allows API authors to declare the\ndesired OpenAPI model packages instead of using the go package path of\nAPI types.\n([kubernetes/kubernetes#131755](https://redirect.github.com/kubernetes/kubernetes/pull/131755),\n[@&#8203;jpbetz](https://redirect.github.com/jpbetz)) \\[SIG API\nMachinery, Apps, Architecture, Auth, CLI, Cloud Provider, Cluster\nLifecycle, Instrumentation, Network, Node, Scheduling, Storage and\nTesting]\n- Support for `kubectl get -o kyaml` is now on by default. To disable\nit, set `KUBECTL_KYAML=false`.\n([kubernetes/kubernetes#133327](https://redirect.github.com/kubernetes/kubernetes/pull/133327),\n[@&#8203;thockin](https://redirect.github.com/thockin)) \\[SIG CLI]\n- The storage version for MutatingAdmissionPolicy is updated to v1beta1.\n([kubernetes/kubernetes#133715](https://redirect.github.com/kubernetes/kubernetes/pull/133715),\n[@&#8203;cici37](https://redirect.github.com/cici37)) \\[SIG API\nMachinery, Etcd and Testing]\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - \"before 8am on Monday\" (UTC),\nAutomerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0Mi43NC41IiwidXBkYXRlZEluVmVyIjoiNDIuNzQuNSIsInRhcmdldEJyYW5jaCI6Im1haW4iLCJsYWJlbHMiOlsiZGVwZW5kZW5jaWVzIl19-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2026-01-19T11:46:28Z",
          "tree_id": "5b1dea8df4cafdb30d91aa76e6283dbb9e3f1228",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/c28577df824da63d5759a149df623c30aa108c09"
        },
        "date": 1768825363641,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "scaling_efficiency_2_cores",
            "value": 0.8609,
            "unit": "",
            "extra": "Scaling efficiency at 2 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "scaling_efficiency_4_cores",
            "value": 0.7992,
            "unit": "",
            "extra": "Scaling efficiency at 4 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "scaling_efficiency_8_cores",
            "value": 0.7734,
            "unit": "",
            "extra": "Scaling efficiency at 8 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "scaling_efficiency_16_cores",
            "value": 0.6029,
            "unit": "",
            "extra": "Scaling efficiency at 16 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "scaling_efficiency_24_cores",
            "value": 0.6146,
            "unit": "",
            "extra": "Scaling efficiency at 24 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "scaling_efficiency_avg",
            "value": 0.7302,
            "unit": "",
            "extra": "Average scaling efficiency across all multi-core tests (1.0 = perfect)"
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
          "id": "2c3976c9672536835e94dae07a4cc7f26333276e",
          "message": "user lowercase for event names (#1816)\n\nhttps://github.com/open-telemetry/otel-arrow/blob/main/rust/otap-dataflow/docs/telemetry/events-guide.md#event-naming\n\nMoving to lowercase. We are not fully following the guided name yet.\nWill tackle that one module at a time in follow ups.\n\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-01-19T12:14:46Z",
          "tree_id": "ed21e6fbb8d8f52aecdf6a40f56b90cb4c53b8e7",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/2c3976c9672536835e94dae07a4cc7f26333276e"
        },
        "date": 1768827294695,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "scaling_efficiency_2_cores",
            "value": 0.8621,
            "unit": "",
            "extra": "Scaling efficiency at 2 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "scaling_efficiency_4_cores",
            "value": 0.8385,
            "unit": "",
            "extra": "Scaling efficiency at 4 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "scaling_efficiency_8_cores",
            "value": 0.7471,
            "unit": "",
            "extra": "Scaling efficiency at 8 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "scaling_efficiency_16_cores",
            "value": 0.5668,
            "unit": "",
            "extra": "Scaling efficiency at 16 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "scaling_efficiency_24_cores",
            "value": 0.6675,
            "unit": "",
            "extra": "Scaling efficiency at 24 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "scaling_efficiency_avg",
            "value": 0.7364,
            "unit": "",
            "extra": "Average scaling efficiency across all multi-core tests (1.0 = perfect)"
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
          "id": "30e7b3e15561011f3f17cb88d4f057849249b58c",
          "message": "chore(deps): update dependency pyarrow to v23 (#1821)\n\nThis PR contains the following updates:\n\n| Package | Change |\n[Age](https://docs.renovatebot.com/merge-confidence/) |\n[Confidence](https://docs.renovatebot.com/merge-confidence/) |\n|---|---|---|---|\n| [pyarrow](https://redirect.github.com/apache/arrow) | `==22.0.0` →\n`==23.0.0` |\n![age](https://developer.mend.io/api/mc/badges/age/pypi/pyarrow/23.0.0?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/pypi/pyarrow/22.0.0/23.0.0?slim=true)\n|\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - \"before 8am on Monday\" (UTC),\nAutomerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0Mi43NC41IiwidXBkYXRlZEluVmVyIjoiNDIuNzQuNSIsInRhcmdldEJyYW5jaCI6Im1haW4iLCJsYWJlbHMiOlsiZGVwZW5kZW5jaWVzIl19-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-01-19T17:24:08Z",
          "tree_id": "ddffb8972a81dc0b3ad16c3d0719449e75ff01cf",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/30e7b3e15561011f3f17cb88d4f057849249b58c"
        },
        "date": 1768845684888,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "scaling_efficiency_2_cores",
            "value": 0.8554,
            "unit": "",
            "extra": "Scaling efficiency at 2 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "scaling_efficiency_4_cores",
            "value": 0.8763,
            "unit": "",
            "extra": "Scaling efficiency at 4 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "scaling_efficiency_8_cores",
            "value": 0.6772,
            "unit": "",
            "extra": "Scaling efficiency at 8 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "scaling_efficiency_16_cores",
            "value": 0.6202,
            "unit": "",
            "extra": "Scaling efficiency at 16 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "scaling_efficiency_24_cores",
            "value": 0.6287,
            "unit": "",
            "extra": "Scaling efficiency at 24 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "scaling_efficiency_avg",
            "value": 0.7316,
            "unit": "",
            "extra": "Average scaling efficiency across all multi-core tests (1.0 = perfect)"
          }
        ]
      }
    ]
  }
}