window.BENCHMARK_DATA = {
  "lastUpdate": 1779157120659,
  "repoUrl": "https://github.com/open-telemetry/otel-arrow",
  "entries": {
    "Benchmark": [
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
          "id": "2d53d89e89d477a39c38d39d863e942167f122ea",
          "message": "docs: document saturation test workload characteristics (#3021)\n\nDocument that saturation tests use static 1KB log bodies with realistic\nentropy (512 unique bodies), distinguishing them from other tests that\nuse semantic_conventions (~300 byte logs). Also removes the stale TODO\nand adds scaling efficiency formula explanation with link to the\nscaling-efficiency benchmark page.",
          "timestamp": "2026-05-18T23:28:48Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/2d53d89e89d477a39c38d39d863e942167f122ea"
        },
        "date": 1779157120125,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.14689917544350198,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.3454237024356081,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 18.241071428571427,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 18.34375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.002115,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle Test Duration"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.7105936585896826,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 1.4307172338602914,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 40.47265625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 41.2109375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.002207,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle Test Duration"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.07859524856599136,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.29283912163214454,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 17.287388392857142,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 17.39453125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.002178,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle Test Duration"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.09695696744583114,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.2476454552535244,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 14.989397321428571,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 15.16796875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.002123,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Test Duration"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.25790844121114315,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.510226512026154,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 21.98046875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 22.36328125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.002217,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle Test Duration"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.35471506142992704,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.7940224159402243,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 29.3515625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 29.5859375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.002246,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle Test Duration"
          }
        ]
      }
    ]
  }
}