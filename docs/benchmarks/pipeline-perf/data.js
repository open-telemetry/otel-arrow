window.BENCHMARK_DATA = {
  "lastUpdate": 1749142645750,
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
          "id": "acd9575ef987f34cf6325e07d12860ba45e2fd19",
          "message": "PerfTest - add more perf metrics to the output for trend tracking (#494)\n\nhttps://github.com/open-telemetry/otel-arrow/pull/491 added a single to\nfigure out the overall flow. This adds few more metrics.\n\nNote: They get stored in a \"benchmarks\" branch in this repo -\nhttps://github.com/open-telemetry/otel-arrow/blob/benchmarks/docs/benchmarks/pipeline-perf/data.js\nThe results are appended to the file after every run (currently\ntriggered when merge to main occurs, will change it later to run nightly\netc.), and we can make a GitHub-Pages site rendering the results in\ncharts.\n\n---------\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2025-05-30T09:13:09-07:00",
          "tree_id": "5e296497cb89f66a9065bc84d4511c852d7a4c31",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/acd9575ef987f34cf6325e07d12860ba45e2fd19"
        },
        "date": 1748621769561,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 468833.3333333333,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 14065000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 14065000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-avg",
            "value": 2.51,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 2.84,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 142.53,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 158.27,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 439500,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 13185000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 13185000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 2.32,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 2.8,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 113.88,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 129.84,
            "unit": "MiB"
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
          "id": "63ef9ec1bf491862d223b3f7fad60a45fbea5645",
          "message": "perf test - switch back to ubuntu runner until new machines are ready (#540)\n\nSwitching perf test back to Ubuntu Github runner for now. It seems that\nthe self-hosted runners are being moved to a [new infra, and is not\nquite ready\nnow](https://github.com/open-telemetry/community/issues/2701#issuecomment-2898521704)",
          "timestamp": "2025-06-05T15:47:49Z",
          "tree_id": "80eb6ef25c53f7cf218c53e36086198582e326d0",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/63ef9ec1bf491862d223b3f7fad60a45fbea5645"
        },
        "date": 1749139079078,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 465000,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 13950000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 13950000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-avg",
            "value": 2.46,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 2.81,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 141.18,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 160.43,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 446666.6666666667,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 13400000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 13400000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 2.41,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 2.81,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 113.26,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 139.05,
            "unit": "MiB"
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
          "distinct": true,
          "id": "019d52aaf8f65c29045365137306bca6f0c993c4",
          "message": "[query-engine] KQL accessor support (#542)\n\n## Changes\n\n* Adds parsing of KQL accessor expressions into query engine\nexpressions.\n\n## Details\n\nI have been working with @drewrelmas on the query engine effort. This is\nmy first PR to start contributing code!\n\nThe goal here is to parse KQL [Dynamic object\naccessors](https://learn.microsoft.com/kusto/query/scalar-data-types/dynamic)\ninto something usable by the query engine.\n\n### Why a new folder?\n\nThe current folder (`query_abstractions`) has a bunch of stuff already.\nI want to merge what @drewrelmas has done into my prototyping efforts. I\ndecided to start a new folder to keep the diff small and hyper-focused.\nThere will be follow-up PRs where we will merge all of this together and\nhave a single thing. This folder should eventually contain a)\nIL\\expression tree for query engine, b) kql & ottl\nparsers\\transpilers(?) for a, and c) engine implementations/prototypes\nwhich can execute those.\n\n---------\n\nCo-authored-by: Drew Relmas <drewrelmas@gmail.com>",
          "timestamp": "2025-06-05T16:34:22Z",
          "tree_id": "6ac281cda34a939b20c007463a3b80c14a33b533",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/019d52aaf8f65c29045365137306bca6f0c993c4"
        },
        "date": 1749141849619,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 460666.6666666667,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 13820000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 13820000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-avg",
            "value": 2.48,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 2.82,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 133.72,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 160.53,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 435166.6666666667,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 13055000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 13055000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 2.36,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 2.79,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 111.81,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 134.51,
            "unit": "MiB"
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
          "distinct": true,
          "id": "4e4c6b8d6088c52db4b0a67b9b93a3292f38e930",
          "message": "chore: add `--all-targets` to `cargo xtask check` (#545)\n\nThe impetus here is so we can see the clippy issues in tests as well\nwith this command",
          "timestamp": "2025-06-05T16:47:33Z",
          "tree_id": "788ca5dd9ee6c7e9b4cbed6ab303afba4729202a",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/4e4c6b8d6088c52db4b0a67b9b93a3292f38e930"
        },
        "date": 1749142643940,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 463000,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 13890000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 13890000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-avg",
            "value": 2.48,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 2.82,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 139.6,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 186.47,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 436166.6666666667,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 13085000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 13085000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 2.35,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 2.79,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 107.69,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 128.09,
            "unit": "MiB"
          }
        ]
      }
    ]
  }
}