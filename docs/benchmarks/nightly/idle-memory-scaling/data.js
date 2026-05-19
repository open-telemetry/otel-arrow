window.BENCHMARK_DATA = {
  "lastUpdate": 1779157123290,
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
        "date": 1779157122764,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_memory_constant_overhead_mib",
            "value": 15.23,
            "unit": "MiB",
            "extra": "Constant memory overhead (C in Memory = C + N*R)"
          },
          {
            "name": "idle_memory_per_core_overhead_mib",
            "value": 0.81,
            "unit": "MiB",
            "extra": "Per-core memory overhead (R in Memory = C + N*R)"
          },
          {
            "name": "idle_memory_r_squared",
            "value": 0.993,
            "unit": "",
            "extra": "Linear fit quality (R²); 1.0 = perfect linear scaling"
          },
          {
            "name": "idle_memory_1core_mib",
            "value": 14.99,
            "unit": "MiB",
            "extra": "Idle memory at 1 core(s); predicted=16.0 MiB, error=7.0%"
          },
          {
            "name": "idle_memory_2core_mib",
            "value": 17.29,
            "unit": "MiB",
            "extra": "Idle memory at 2 core(s); predicted=16.8 MiB, error=2.5%"
          },
          {
            "name": "idle_memory_4core_mib",
            "value": 18.24,
            "unit": "MiB",
            "extra": "Idle memory at 4 core(s); predicted=18.5 MiB, error=1.2%"
          },
          {
            "name": "idle_memory_8core_mib",
            "value": 21.98,
            "unit": "MiB",
            "extra": "Idle memory at 8 core(s); predicted=21.7 MiB, error=1.3%"
          },
          {
            "name": "idle_memory_16core_mib",
            "value": 29.35,
            "unit": "MiB",
            "extra": "Idle memory at 16 core(s); predicted=28.2 MiB, error=4.0%"
          },
          {
            "name": "idle_memory_32core_mib",
            "value": 40.47,
            "unit": "MiB",
            "extra": "Idle memory at 32 core(s); predicted=41.1 MiB, error=1.6%"
          }
        ]
      }
    ]
  }
}