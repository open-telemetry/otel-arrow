window.BENCHMARK_DATA = {
  "lastUpdate": 1768541690948,
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
      }
    ]
  }
}