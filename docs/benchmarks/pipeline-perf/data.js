window.BENCHMARK_DATA = {
  "lastUpdate": 1748621456492,
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
          "id": "bcdf4b6d1771e1996ca63d00cf8e5ba2240d92b9",
          "message": "PerfTest - keep track of perf results using gh action (#491)\n\nDraft to figure out permissions etc.",
          "timestamp": "2025-05-29T15:46:24-07:00",
          "tree_id": "841b13867eb70b84c488b12512f44e1ebc68790d",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/bcdf4b6d1771e1996ca63d00cf8e5ba2240d92b9"
        },
        "date": 1748558951296,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 435666.6666666667,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 462833.3333333333,
            "unit": "logs/sec"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "8164192+clhain@users.noreply.github.com",
            "name": "clhain",
            "username": "clhain"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "e79896a405cb22d6b6adecc129777d0d246a9612",
          "message": "Perf - support hook execution and runtime data on test framework elements. (#493)\n\nThis change refactors hook and runtime classes and adds support for them\non Test Elements (Test Suite, Test, Test Step). This will allow us to\ne.g. run global hooks in order to e.g. deploy infrastructure without\nhaving to tie it to a particular component. Change also includes\nrestructuring to provide more uniformity between component and test\nelement naming, and introduces base classes for test elements and their\nassociated context classes.\n\n- Rename LifecycleComponent -> Component\n- Rename LifecyclePhase -> ComponentPhase\n- Rename HookableLifecyclePhase -> HookableComponentPhase\n- Rename LifecycleHookContext -> ComponentHookContext\n- Move ComponentHookContext, HookableComponentPhase -> lib.core.context\n- Rename ComponentRuntime -> Runtime \n- Move Runtime to lib.core.runtime (new).\n- Add TestFrameworkElement as a base for all Test Element classes\n(TestSuite, TestDefinition, TestStep)\n- Added TestFrameworkElementContext as a base for all Test Element\nrelated contexts.\n- Added TestElementHookContext, HookableTestPhase in lib.core.context\n\nThis should be the last major change to the core. Config factory to\nfollow, then various strategy implementations. #467",
          "timestamp": "2025-05-30T09:07:59-07:00",
          "tree_id": "51638282d1a3464f8707a0519c105404f29f211a",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/e79896a405cb22d6b6adecc129777d0d246a9612"
        },
        "date": 1748621454871,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 466000,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 438833.3333333333,
            "unit": "logs/sec"
          }
        ]
      }
    ]
  }
}