window.BENCHMARK_DATA = {
  "lastUpdate": 1768586085352,
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
      }
    ]
  }
}