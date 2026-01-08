window.BENCHMARK_DATA = {
  "lastUpdate": 1767897796410,
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
      }
    ]
  }
}