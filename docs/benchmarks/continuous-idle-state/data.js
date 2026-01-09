window.BENCHMARK_DATA = {
  "lastUpdate": 1767981776416,
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
          "id": "b524eb10a8f31a1eff89878cbf2ce6fd9fde7d5b",
          "message": "PipelinePerfTest - add test for idle state numbers (#1740)\n\nRuns for few seconds only to avoid taking up perf test machine.\nThis test maybe okay to be run in normal GH runners and we can run for\nmore duration. For future considerations...",
          "timestamp": "2026-01-08T18:12:47Z",
          "tree_id": "393d812e9cb9104bbc922a56b1e503be7e13925b",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/b524eb10a8f31a1eff89878cbf2ce6fd9fde7d5b"
        },
        "date": 1767897796011,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.05092052091183608,
            "unit": "%",
            "extra": "Continuous - Idle State Performance/Idle State Baseline - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.06625782070254693,
            "unit": "%",
            "extra": "Continuous - Idle State Performance/Idle State Baseline - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 27.246651785714285,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance/Idle State Baseline - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 27.34375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance/Idle State Baseline - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.000889,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance/Idle State Baseline - Idle Test Duration"
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
          "id": "7cffafe2cb2c3ac605852d8d87ba77b4a41b716c",
          "message": "Internal Telemetry Guidelines (#1727)\n\nThis PR defines a set of guidelines for our internal telemetry and for\ndescribing how we can establish a telemetry by design process.\n\nOnce this PR is merged, I will follow up with a series of PRs to align\nthe existing instrumentation with these recommendations.\n\n---------\n\nCo-authored-by: Cijo Thomas <cithomas@microsoft.com>",
          "timestamp": "2026-01-08T22:23:58Z",
          "tree_id": "23f046b4d5bf8c73ed3a81efab1b6611ecccbeb9",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/7cffafe2cb2c3ac605852d8d87ba77b4a41b716c"
        },
        "date": 1767913216997,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.06261100518710624,
            "unit": "%",
            "extra": "Continuous - Idle State Performance/Idle State Baseline - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.08300101301332502,
            "unit": "%",
            "extra": "Continuous - Idle State Performance/Idle State Baseline - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 27.46875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance/Idle State Baseline - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 27.6328125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance/Idle State Baseline - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.00068,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance/Idle State Baseline - Idle Test Duration"
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
          "id": "155b3c454d5a609a47ec4df318252690c617349e",
          "message": "PerfTest - Idle run perf numbers for all available cores (#1744)\n\nWe were running with core-id=0, so df_engine only spun one thread. \nAdding one more test where we don't restrict cores, so it'll run on all\ncores (creating as many threads) and show memory/cpu.\n\n(the lab perf machine has 128 cores)",
          "timestamp": "2026-01-09T17:32:51Z",
          "tree_id": "826543e921ea6cf2cfefb2b38031cfd0ba1555e6",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/155b3c454d5a609a47ec4df318252690c617349e"
        },
        "date": 1767981776063,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_cpu_percentage_avg",
            "value": 2.3018591081175344,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 2.469550219455253,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 620.1462053571429,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 621.015625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.007306,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle Test Duration"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.06273238060582037,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.08489199066147861,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 27.76171875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 27.93359375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.000921,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Test Duration"
          }
        ]
      }
    ]
  }
}