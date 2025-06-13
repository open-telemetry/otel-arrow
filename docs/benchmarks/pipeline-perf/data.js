window.BENCHMARK_DATA = {
  "lastUpdate": 1749842422521,
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
      },
      {
        "commit": {
          "author": {
            "email": "drewrelmas@gmail.com",
            "name": "Drew Relmas",
            "username": "drewrelmas"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "eecc100a7309eed8f761e57072095b9c87e98633",
          "message": "[Editorial] Housekeeping updates to query_abstraction and query_engine subfolders (#547)\n\nAccomplishes the following:\n1. Enabled CI for new `query_engine` subfolder.\n2. Updates `query_abstraction` and `query_engine` to use workspace\nconfiguration in their `Cargo.toml`s to reduce duplication, similar to\nthe other rust workspaces in the repository.\n3. Minor edits to initial query_engine commit for compliance with `fmt`\nand `clippy`.",
          "timestamp": "2025-06-05T20:40:53Z",
          "tree_id": "a6477ebb13537c0f294a82d203d1d1c4c335f7cf",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/eecc100a7309eed8f761e57072095b9c87e98633"
        },
        "date": 1749156642655,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 455833.3333333333,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 13675000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 13675000,
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
            "value": 2.84,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 143.84,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 177.59,
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
            "value": 2.38,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 2.8,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 106.44,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 131.13,
            "unit": "MiB"
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
          "id": "5184877e3fe1d2efa6a2bfd15c40f9b96f146d6e",
          "message": "chore(deps): update ossf/scorecard-action digest to 05b42c6 (#549)\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n|---|---|---|---|\n|\n[ossf/scorecard-action](https://redirect.github.com/ossf/scorecard-action)\n| action | digest | `f49aabe` -> `05b42c6` |\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - At any time (no schedule defined),\nAutomerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0MC40MC4zIiwidXBkYXRlZEluVmVyIjoiNDAuNDAuMyIsInRhcmdldEJyYW5jaCI6Im1haW4iLCJsYWJlbHMiOlsiZGVwZW5kZW5jaWVzIl19-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2025-06-05T21:13:46Z",
          "tree_id": "6aa2b1b540e4300125fe3e2f9f1926f2090f261b",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/5184877e3fe1d2efa6a2bfd15c40f9b96f146d6e"
        },
        "date": 1749158609187,
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
            "value": 2.49,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 2.85,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 145.53,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 165.59,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 439000,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 13170000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 13170000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 2.38,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 2.79,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 103.69,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 118.83,
            "unit": "MiB"
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
          "distinct": false,
          "id": "a921de10ac7e956db8eb82a1a7936daa172677a9",
          "message": "Perf - Add otel based classes for interacting with metric / trace data in the orchestrator (#541)\n\nThis PR adds the basic plumbing for a move towards a common otel-based\nmechanism to allow test framework components and plugins to generate,\nstore, and access telemetry data.\n\nThe basic idea is:\n1. Test framework elements register standard otel Metric/Span exporters\non creation (log exporter too but no plan to support native store /\nquery of these... except maybe as span events)\n2. By default, we register exporters that will store these signals in\nmemory (but remote backends via otlp should work fine too)\n3. TestFrameworkElements generate spans / events via otel sdk for\n'everything' (tests, steps, hooks, etc)\n4. Plugins can generate their own spans/metrics the same way (e.g.\ndocker process monitor plugin creates a gauge container.cpu.utilization\nusing the framework's ctx.get_meter())\n5. signal retriever interfaces provide access to stored data (default in\nmemory, or remote) represented as a MetricDataFrame, SpanDataFrame,\nSpanEventDataFrame... signal specific pd.DataFrames - simplified\nrepresentations of the otel dataclasses (felt a lot easier to work with\nthan just a raw list[Span] / MetricData) - better support for storing to\ndisk.\n\nExample use:\n1. Docker Container Monitoring plugin is started and begins collecting\ncpu data for containers every X seconds\n2. Load is started and allowed to come to a steady state (which is\noptionally validated via metricsretriver provided data)\n3. Framework automatically generates a span for \"Test Step: Observe\nProcess\", a 60 second pause\n5. A reporting plugin is configured to fetch the span for this test step\n(or a custom event added via a hook insertion, or etc...), and then\nqueries the metricsretriever for all container.cpu.utilization metrics\nbetween the span.start and span.end - aggregates them, and reports that\nvalue (optionally as it's own event or metric!)\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2025-06-05T21:20:25Z",
          "tree_id": "59ad32679eb816f10a331a568ec2f27d03ed0e62",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/a921de10ac7e956db8eb82a1a7936daa172677a9"
        },
        "date": 1749159016621,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 461666.6666666667,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 13850000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 13850000,
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
            "value": 2.79,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 146.33,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 179.88,
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
            "value": 2.39,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 2.8,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 111.94,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 132.6,
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
          "distinct": false,
          "id": "ac1d1c945a75208a712381f71c0f10fa709cd7b6",
          "message": "feat: support metrics and traces in parquet exporter (#544)\n\nCompletes: #503 \n\nThe bulk of the important changes here are in the `idgen` module, which\nwas hard-coded to encode IDs for logs. The majority of the rest of the\nchanges are to testing code (including the code that generates record\nbatches for the tests).\n\nI also removed the unused `sort_by_parent_id`. This was added for some\nexperimental optimization to AttributeStore. We can add this back in the\nfuture if we decide we need it.\n\n---------\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2025-06-05T21:28:41Z",
          "tree_id": "223e1a87994cfee09443dae4df1a969c5191b7ef",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/ac1d1c945a75208a712381f71c0f10fa709cd7b6"
        },
        "date": 1749159497575,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 464500,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 13935000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 13935000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-avg",
            "value": 2.49,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 2.82,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 138.06,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 156.29,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 440333.3333333333,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 13210000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 13210000,
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
            "value": 2.82,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 116.65,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 135.61,
            "unit": "MiB"
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
          "id": "36be181606219d3d6436ab984d06864d1280ad9e",
          "message": "chore(deps): update golang:1.24 docker digest to db5d0af (#550)\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n|---|---|---|---|\n| golang | stage | digest | `81bf592` -> `db5d0af` |\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - At any time (no schedule defined),\nAutomerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0MC40MC4zIiwidXBkYXRlZEluVmVyIjoiNDAuNDAuMyIsInRhcmdldEJyYW5jaCI6Im1haW4iLCJsYWJlbHMiOlsiZGVwZW5kZW5jaWVzIl19-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2025-06-05T21:35:13Z",
          "tree_id": "2394a3f6861dd1b66f746ff572a9342a7bf376a8",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/36be181606219d3d6436ab984d06864d1280ad9e"
        },
        "date": 1749159914098,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 471000,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 14130000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 14130000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-avg",
            "value": 2.52,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 2.84,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 140.58,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 162.61,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 443166.6666666667,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 13295000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 13295000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 2.39,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 2.83,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 111.44,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 136.97,
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
          "id": "9a44f8903699eb226278318fba615819dd37267e",
          "message": "[query-engine] KQL bool, double, & real (#551)\n\n## Changes\n\n* Adds parsing of KQL bool & double literals and the real expression\n\n---------\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2025-06-05T22:58:42Z",
          "tree_id": "9453f1038b4a17190276b54906d8131605e48024",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/9a44f8903699eb226278318fba615819dd37267e"
        },
        "date": 1749164940456,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 461333.3333333333,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 13840000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 13840000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-avg",
            "value": 2.47,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 2.83,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 138.81,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 170.42,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 441666.6666666667,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 13250000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 13250000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 2.37,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 2.79,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 109.58,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 141.27,
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
          "distinct": false,
          "id": "2cc68c9a976b6ef1a76eeb38b058252422ab0eff",
          "message": "[query-engine] KQL bool, double, & real - linting (#553)\n\n## Changes\n\n* Clean up clippy warnings from #551",
          "timestamp": "2025-06-06T15:25:40Z",
          "tree_id": "cb0e04aabb6be15b80d9f5cdfaae27107ac6eb76",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/2cc68c9a976b6ef1a76eeb38b058252422ab0eff"
        },
        "date": 1749224135225,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 467500,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 14025000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 14025000,
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
            "value": 141.37,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 164.79,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 441000,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 13230000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 13230000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 2.38,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 2.81,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 112.77,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 143,
            "unit": "MiB"
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
          "id": "98c62a3a1e97bf12a2c97515e3d3035a3c7f5a41",
          "message": "Perf - Add otel supporting functionality to framework context objects (#552)\n\nThe PR adds support for tracing / logging / metrics in the various\nframework context objects.\n\nE.g. automatic creation / end of spans with status, ability to fetch\notel tracers, meters, and logging handler inside plugins, decorating\nspans with ctx metadata, and generating span events.\n\n---------\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2025-06-06T15:45:01Z",
          "tree_id": "c6bbde4b9fd0cd012d9f6061242d4a62a50f90ce",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/98c62a3a1e97bf12a2c97515e3d3035a3c7f5a41"
        },
        "date": 1749225305024,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 465500,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 13965000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 13965000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-avg",
            "value": 2.49,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 2.83,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 143.41,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 177.38,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 437000,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 13110000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 13110000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 2.3,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 2.77,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 112.47,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 137.73,
            "unit": "MiB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "david@ddahl.com",
            "name": "David Dahl",
            "username": "daviddahl"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "4caab9290379efb0f4d0782f71d4fb1a61eb23c2",
          "message": "Add batch_processor and tests (#347)\n\nA batch_processor WIP\n\n---------\n\nCo-authored-by: David Dahl <daviddahl@users.noreply.github.com>\nCo-authored-by: Laurent Quérel <l.querel@f5.com>\nCo-authored-by: Utkarsh Umesan Pillai <66651184+utpilla@users.noreply.github.com>",
          "timestamp": "2025-06-06T17:48:05Z",
          "tree_id": "53f3f158b3546ac62593d2c3e713375db25c855d",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/4caab9290379efb0f4d0782f71d4fb1a61eb23c2"
        },
        "date": 1749232688245,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 469833.3333333333,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 14095000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 14095000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-avg",
            "value": 2.47,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 2.8,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 140.79,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 172.7,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 448833.3333333333,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 13465000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 13465000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 2.33,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 2.81,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 105.63,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 122.3,
            "unit": "MiB"
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
          "id": "c74b7018920c39925339a4ea9dbfa9f2afc0a856",
          "message": "Perf - remaining core telemetry support (#554)\n\nThis adds the actual logging and span instrumentation to core test\ncomponents, and introduces centralized plugin error handling. Misc\nconfig base models, and other accumulated nit/todo fixes as well.\n\n---------\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2025-06-06T20:50:22Z",
          "tree_id": "686068ac2dc32161967f962ea66ba3920c7a1a5e",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/c74b7018920c39925339a4ea9dbfa9f2afc0a856"
        },
        "date": 1749243623204,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 470000,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 14100000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 14100000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-avg",
            "value": 2.5,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 2.83,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 155.86,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 188.11,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 444166.6666666667,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 13325000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 13325000,
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
            "value": 109.28,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 124.84,
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
          "distinct": false,
          "id": "cb49db0d0eaa6a123e7a9864c60c8b89997ec6bf",
          "message": "[query-engine] KQL datetime (#556)\n\n## Changes\n\n* Adds parsing of KQL datetime expression",
          "timestamp": "2025-06-09T15:29:19Z",
          "tree_id": "030ba9ba290218124362c55e63fa3ab012c5e9e0",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/cb49db0d0eaa6a123e7a9864c60c8b89997ec6bf"
        },
        "date": 1749483561202,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 458166.6666666667,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 13745000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 13745000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-avg",
            "value": 2.45,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 2.81,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 137.65,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 160.56,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 438166.6666666667,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 13145000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 13145000,
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
            "value": 118.32,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 144.29,
            "unit": "MiB"
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
          "id": "060f7bcc572831e4ab2f9783c392c79831c7e9ce",
          "message": "chore(deps): update dependency go to v1.24.4 (#518)\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n|---|---|---|---|\n| [go](https://redirect.github.com/actions/go-versions) | uses-with |\npatch | `1.24.3` -> `1.24.4` |\n\n---\n\n### Release Notes\n\n<details>\n<summary>actions/go-versions (go)</summary>\n\n###\n[`v1.24.4`](https://redirect.github.com/actions/go-versions/releases/tag/1.24.4-15482447176):\n1.24.4\n\n[Compare\nSource](https://redirect.github.com/actions/go-versions/compare/1.24.3-14875263452...1.24.4-15482447176)\n\nGo 1.24.4\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - \"before 8am every weekday\" (UTC),\nAutomerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0MC4zMy42IiwidXBkYXRlZEluVmVyIjoiNDAuNDAuMyIsInRhcmdldEJyYW5jaCI6Im1haW4iLCJsYWJlbHMiOlsiZGVwZW5kZW5jaWVzIl19-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2025-06-09T15:32:15Z",
          "tree_id": "a0eb8911a703609eaf892ffa924da95fdff36d12",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/060f7bcc572831e4ab2f9783c392c79831c7e9ce"
        },
        "date": 1749483871587,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 471166.6666666667,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 14135000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 14135000,
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
            "value": 2.81,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 153.66,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 176.36,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 443500,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 13305000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 13305000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 2.38,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 2.81,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 115.62,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 143.63,
            "unit": "MiB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "drewrelmas@gmail.com",
            "name": "Drew Relmas",
            "username": "drewrelmas"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "d3198843a30d39600d12a2424492990812b98ba9",
          "message": "Release v0.36.0 (#564)\n\nPrimary motivation for this release (besides the fact it has been 2\nmonths since preceeding one) is confirming code movement performed in\n#413 doesn't adversely affect our ability to update our Golang\ncomponents.\n\n# Changelog\n\n- Remove `concurrentbatch` and `obfuscation` processors.\n[#409](https://github.com/open-telemetry/otel-arrow/pull/409)\n- OTAP AttributeStore parent_id encoding cleanup.\n[#431](https://github.com/open-telemetry/otel-arrow/pull/431)\n- Upgrade Go to 1.24.3.\n[#440](https://github.com/open-telemetry/otel-arrow/pull/440),\n[#508](https://github.com/open-telemetry/otel-arrow/pull/508)\n- Fix time unit of `DurationTimeUnixNano` to `Duration_ns` in Traces.\n[#517](https://github.com/open-telemetry/otel-arrow/pull/517)\n- Upgrade to v0.127.0 / v1.33.0 of collector dependencies.\n[#526](https://github.com/open-telemetry/otel-arrow/pull/526)",
          "timestamp": "2025-06-09T21:09:06Z",
          "tree_id": "dbb577ba2781041a76cb256ecfe3718883e8900d",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/d3198843a30d39600d12a2424492990812b98ba9"
        },
        "date": 1749503950360,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 469500,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 14085000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 14085000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-avg",
            "value": 2.49,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 2.83,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 150.68,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 182.32,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 443833.3333333333,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 13315000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 13315000,
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
            "value": 2.78,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 108.75,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 128.69,
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
          "id": "3be0cfe048086576ee5a101a09d581063621eac7",
          "message": "[query-engine] Engine implementation folders (#563)\n\n## Changes\n\n* Define folders/crates to contain engine implementations",
          "timestamp": "2025-06-09T22:09:48Z",
          "tree_id": "7e32947fe8b2bd23e1361c71bb3344dca1a481c2",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/3be0cfe048086576ee5a101a09d581063621eac7"
        },
        "date": 1749507604976,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 470833.3333333333,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 14125000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 14125000,
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
            "value": 141.96,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 171.25,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 445333.3333333333,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 13360000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 13360000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 2.3,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 2.81,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 104.73,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 115.43,
            "unit": "MiB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "drewrelmas@gmail.com",
            "name": "Drew Relmas",
            "username": "drewrelmas"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "b73f4de67a16da384f420d98f6d67b49e877db99",
          "message": "Revert Go module subfolder move and bump to v0.37.0 (#567)\n\nNext steps to resolving the failed `v0.36.0` release mentioned at #566 \n- Revert folder change from #413\n- Add `retract` directive for `github.com/open-telemetry/otel-arrow\nv0.36.0`\n- Bump version number to `v0.37.0` for a clean release with the\nretraction",
          "timestamp": "2025-06-10T16:00:59Z",
          "tree_id": "570627a84fdfa1f2efab1f8c7bdc21d367a5bd77",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/b73f4de67a16da384f420d98f6d67b49e877db99"
        },
        "date": 1749571860225,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 467000,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 14010000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 14010000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-avg",
            "value": 2.5,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 2.85,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 144.16,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 162.82,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 441166.6666666667,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 13235000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 13235000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 2.31,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 2.81,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 104.95,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 122.98,
            "unit": "MiB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "drewrelmas@gmail.com",
            "name": "Drew Relmas",
            "username": "drewrelmas"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "5afc0bc560667bf517043cc47774a9d1b9257b78",
          "message": "Downgrade Go to 1.23.0 for Collector-Contrib compatibility (#569)\n\nAlthough our latest release `v0.37.0` resolved problems mentioned in\n#566 caused by folder structure changes, it has another unrelated issue.\n\nThe Go version used by our modules was upgraded to `1.24.x` in #440 and\n#508 due to prompting from the recently enabled Renovate bot. However,\nthe primary usage of the modules is in the\nOpenTelemetry-Collector-Contrib repository which currently only supports\ncomponents adhering to `1.23.0`.\n\nWhile there is nothing functionally wrong with our release `v0.37.0`, it\nis unusable in Collector-Contrib. Therefore, this PR:\n- Downgrades Go to 1.23.0\n- Adjusts Renovate config to constrain Go to that version, similar to\n[Collector-Contrib](https://github.com/open-telemetry/opentelemetry-collector-contrib/blob/5accecf1943f682b9e091fb4bf9e811f7b7fad98/renovate.json#L7-L9)",
          "timestamp": "2025-06-10T17:09:08Z",
          "tree_id": "7855af8d9bc24510c4a81cabf2215deec793df81",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/5afc0bc560667bf517043cc47774a9d1b9257b78"
        },
        "date": 1749575958119,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 466666.6666666667,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 14000000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 14000000,
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
            "value": 2.83,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 148.23,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 180.27,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 442166.6666666667,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 13265000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 13265000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 2.33,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 2.79,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 110.75,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 129.61,
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
          "id": "8677712e3654c367deaaa2afbbaa152009b2cddb",
          "message": "feat: support additional types for adaptive array builders (#562)\n\nImplements adaptive array builder for additional types:\n- Primitive (e.g. Uint, Int and Float)\n- Boolean\n- Binary\n- FixedSizeBinary\n\nThere are some changes to the base traits that may warrant some\nexplanation:\n- Because FixedSizeBinary can return an error when appending a value\n(e.g. if the passed byte array length is wrong), we need to have a\nversion of `append_value` that can return an error. Ideally, we wouldn't\nforce callers to handle the error when the append for a given type can't\nreturn an error. For this reason, there's now two traits for append:\n`ArrayAppend` and `CheckedArrayAppend` and for the latter,\n`append_value` can return a `Result<(), ArrowError>`\n- Something similar has happened in dictionary.rs for appending to\ndicts.\n- The `finish` method on the `ArrayBuilder` trait now just returns an\n`ArrayRef` rather than `ArrayWithType`, which was unnecessary because\n`ArrayRef` already has a `data_type()` method\n- Since some array builders might take additional arguments in the\nconstructor (e.g. for FixedSizeBinary the constructor takes the\nbyte_length argument), the `ArrayBuilderConstructor` trait is now\ngeneric over the args type and so is `AdaptiveArrayBuilder`, which owns\nan instance of the `Args` type and uses this to create the builders.\n- The implication here is that now `AdaptiveArrayBuilder` is responsible\nfor converting from dict builder to native builder, whereas before it\nwas the `MaybeDictionaryBuilder` enum doing this. This change was made\nbecause the arg value (which `MaybeDictionaryBuilder` doesn't have) is\nneeded to do the conversion.\n\nThis PR also tries to clean up where the trait bounds are defined\n(especially in array.rs) so that there's only restrictions added to\ngeneric types in impl/fn blocks where the trait bound is actually\nneeded. This removes some unnecessary code duplication, but more\nimportantly it makes this code much easier to change.\n\n---------\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2025-06-10T20:29:13Z",
          "tree_id": "7cd8bde963e4421ec12a2eadac2bd78e380f7b62",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/8677712e3654c367deaaa2afbbaa152009b2cddb"
        },
        "date": 1749587953066,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 468500,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 14055000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 14055000,
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
            "value": 2.81,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 142.52,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 166.92,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 444166.6666666667,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 13325000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 13325000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 2.3,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 2.8,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 113.03,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 145.28,
            "unit": "MiB"
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
          "id": "a987e15f56b82353cd68905ca5f6e3ba5cbd540a",
          "message": "chore(deps): update rust crate mimalloc-rust to 0.2.0 (#558)\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n|---|---|---|---|\n| [mimalloc-rust](https://redirect.github.com/lemonhx/mimalloc-rust) |\ndev-dependencies | minor | `0.1.5` -> `0.2.0` |\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - \"before 8am on Monday\" (UTC),\nAutomerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0MC40MC4zIiwidXBkYXRlZEluVmVyIjoiNDAuNDAuMyIsInRhcmdldEJyYW5jaCI6Im1haW4iLCJsYWJlbHMiOlsiZGVwZW5kZW5jaWVzIl19-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2025-06-10T20:36:40Z",
          "tree_id": "af4bd55fdd73ae0b5b878f17b589a0ddd9c86f78",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/a987e15f56b82353cd68905ca5f6e3ba5cbd540a"
        },
        "date": 1749588409378,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 469833.3333333333,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 14095000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 14095000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-avg",
            "value": 2.49,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 2.84,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 142.38,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 172.58,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 446000,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 13380000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 13380000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 2.37,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 2.79,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 116.37,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 132.01,
            "unit": "MiB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "drewrelmas@gmail.com",
            "name": "Drew Relmas",
            "username": "drewrelmas"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "0cc25b2631aadeb00a619221f16f8b135ff8d141",
          "message": "Mark terminal version of otel-arrow module and bump to 0.38.0 (#572)\n\nNext step of https://github.com/open-telemetry/otel-arrow/issues/566\ninvolves:\n- Retract `v0.37.0` due to incompatibility with consumers\n(OpenTelemetry-Collector-Contrib)\n- Mark terminal version of `github.com/open-telemetry/otel-arrow` module\nwith deprecation notice\n- Bump version to `v0.38.0`",
          "timestamp": "2025-06-10T21:06:37Z",
          "tree_id": "1e86ec98a852d4c375a3ab2701ff6a71c93ad81f",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/0cc25b2631aadeb00a619221f16f8b135ff8d141"
        },
        "date": 1749590187071,
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
            "value": 138.27,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 164.57,
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
            "value": 2.34,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 2.79,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 115.13,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 142.03,
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
          "id": "71998478c84a9a87185f7a88ba26441e14e6ad25",
          "message": "[query-engine] Add execution context & pipeline expression to recordset engine (#575)\n\n## Changes\n\n* Adds ExecutionContext, Error, Summary, PipelineExpression and helper\nmacros to the recordset engine implementation",
          "timestamp": "2025-06-10T23:27:35Z",
          "tree_id": "72edde13c46bebcc15b47a4f64cf26db02cb6bdc",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/71998478c84a9a87185f7a88ba26441e14e6ad25"
        },
        "date": 1749598650102,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 467500,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 14025000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 14025000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-avg",
            "value": 2.49,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 2.82,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 143.23,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 166.05,
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
            "value": 2.31,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 2.82,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 108.25,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 128.71,
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
          "id": "c88f17432ee55e716e3e89d92faa4e7848925d6b",
          "message": "[query-engine] Add support for parsing KQL logical expressions (#578)\n\n## Changes\n\n* Adds support for parsing KQL logical expressions into query engine\nexpressions",
          "timestamp": "2025-06-11T17:01:25Z",
          "tree_id": "ef05b6ff52899c5f59da071827d037910d8bf7e4",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/c88f17432ee55e716e3e89d92faa4e7848925d6b"
        },
        "date": 1749661887935,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 465666.6666666667,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 13970000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 13970000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-avg",
            "value": 2.47,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 2.82,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 136.52,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 161.46,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 430333.3333333333,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 12910000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 12910000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 2.31,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 2.77,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 114.09,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 133.18,
            "unit": "MiB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "drewrelmas@gmail.com",
            "name": "Drew Relmas",
            "username": "drewrelmas"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "2da489e3219ecef755bdd451745d2f40e8851b56",
          "message": "Remove randomness in go tests to address flaky codecov (#555)\n\nAttempt at addressing #529\n\nAccording to various [codecov\nreadings](https://app.codecov.io/gh/open-telemetry/otel-arrow/pull/554/indirect-changes),\nthe discrepancy in our codecov is often in various error conditions of\ndifferent `pkg/otel` files.\n\n\n![image](https://github.com/user-attachments/assets/66e6b96a-3449-4227-bb3c-a4f2ac07fa69)\n\nThis PR was a result of a brief look into the test infrastructure there\nand noting several usages of `rng` with a seed of type `rand.Uint64`.\nOut of curiosity I removed them and ran checked coverage multiple times\nlocally, and found it stabilizes at reproducible numbers.\n\nI am not entirely sure how important true entropy is in testing the go\nmodules compared to seeded entropy, looking for insights from @jmacd and\n@lquerel who wrote most of it.",
          "timestamp": "2025-06-11T17:15:05Z",
          "tree_id": "d624715db99bc804b44b7f6171ceafd6be572f79",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/2da489e3219ecef755bdd451745d2f40e8851b56"
        },
        "date": 1749662767223,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 460000,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 13800000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 13800000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-avg",
            "value": 2.47,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 2.79,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 146.54,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 166.26,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 442000,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 13260000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 13260000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 2.42,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 2.84,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 107.15,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 126.48,
            "unit": "MiB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "albert.lockett@gmail.com",
            "name": "albertlockett",
            "username": "albertlockett"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "48502a8dc36935326bc309c83c8361429eb857dc",
          "message": "feat: Adaptive array builders for struct & time types (#573)\n\nAdds adaptive array builder implementations for `Struct`,\n`TimestampNano` and `DurationNano`.\n\nSome small cleanup items:\n- changes the `append_value_checked` method on `AdaptiveArrayBuilder` to\njust be `append_value` by having `AdaptiveArrayBuilder` implement the\n`CheckedArrayAppend` trait. This makes the method call more similar to\nhow it's called on the underlying arrow builders (e.g. for\nFixedSizeBinaryBuilder, the method is just called `append_value`\nhttps://docs.rs/arrow/latest/arrow/array/struct.FixedSizeBinaryBuilder.html#method.append_value).\n- removes som unnecessary `'static` lifetime constraints in the\n`array::test` module.\n- made some methods on `AdaptiveBooleanBuilder` public which should not\nhave been private",
          "timestamp": "2025-06-11T18:03:45Z",
          "tree_id": "463bcaeced9c410e980cd4c0ecf82cd623efc11e",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/48502a8dc36935326bc309c83c8361429eb857dc"
        },
        "date": 1749665624731,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 475833.3333333333,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 14275000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 14275000,
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
            "value": 2.83,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 144.21,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 171.45,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 441500,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 13245000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 13245000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 2.29,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 2.8,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 110.67,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 133.11,
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
          "distinct": false,
          "id": "78e8279cf008b606cf57f4b1bd778bfa48ad53ff",
          "message": "[query-engine] Add AnyValue enum to recordset engine (#582)\n\n## Changes\n\n* Define `AnyValue` enum to expose primitives in recordset query engine",
          "timestamp": "2025-06-11T20:21:40Z",
          "tree_id": "0c0ae7b7198eae697c3f6ccd1b44dabb1b8d1fea",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/78e8279cf008b606cf57f4b1bd778bfa48ad53ff"
        },
        "date": 1749673893588,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 465500,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 13965000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 13965000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-avg",
            "value": 2.49,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 2.82,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 144.24,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 172.33,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 433166.6666666667,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 12995000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 12995000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 2.3,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 2.8,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 110.78,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 142.48,
            "unit": "MiB"
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
          "id": "005ea885155259f2978c1644fb12b8cecd00ab82",
          "message": "chore(deps): update python:3.13-slim docker digest to 9ed09f7 (#577)\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n|---|---|---|---|\n| python | final | digest | `d97b595` -> `9ed09f7` |\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - At any time (no schedule defined),\nAutomerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0MC40OC41IiwidXBkYXRlZEluVmVyIjoiNDAuNDguNSIsInRhcmdldEJyYW5jaCI6Im1haW4iLCJsYWJlbHMiOlsiZGVwZW5kZW5jaWVzIl19-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2025-06-11T20:27:41Z",
          "tree_id": "93dd3c71b0ec32237ddea66fa26e0187ebb22ebf",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/005ea885155259f2978c1644fb12b8cecd00ab82"
        },
        "date": 1749674272826,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 471833.3333333333,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 14155000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 14155000,
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
            "value": 2.81,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 142.8,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 168.99,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 441500,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 13245000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 13245000,
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
            "value": 111.5,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 125.47,
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
          "id": "acf0438b52673d3b1e0e2ffab197e6a8de68ea57",
          "message": "[query-engine] Static scalar expressions (#584)\n\n## Changes\n\n* Refactor static scalar expressions into their own enum",
          "timestamp": "2025-06-11T20:40:38Z",
          "tree_id": "ab2738d437181d979d24b3d95d5d74619d9ff90c",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/acf0438b52673d3b1e0e2ffab197e6a8de68ea57"
        },
        "date": 1749675056314,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 460833.3333333333,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 13825000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 13825000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-avg",
            "value": 2.49,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 2.83,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 134.3,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 155.09,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 433333.3333333333,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 13000000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 13000000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 2.3,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 2.78,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 114.32,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 142.39,
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
          "id": "7a5e578d3596de6f22ae42a76c9767c5d233d831",
          "message": "[query-engine] Add data record and batch definitions to recordset engine (#585)\n\n## Changes\n\n* Defines data record and data record batch in the recordset query\nengine",
          "timestamp": "2025-06-11T22:07:23Z",
          "tree_id": "fe6feed4d5ca7448889ff29ae273b25bda962ba4",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/7a5e578d3596de6f22ae42a76c9767c5d233d831"
        },
        "date": 1749680236775,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 467000,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 14010000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 14010000,
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
            "value": 143.93,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 173.82,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 441166.6666666667,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 13235000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 13235000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 2.37,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 2.78,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 117.74,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 151.68,
            "unit": "MiB"
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
          "distinct": true,
          "id": "322e82a2f7edabe612ad372ee0887eb650c13198",
          "message": "[otel-arrow-rust] Add UDP support (#568)\n\n## Changes\n- Add UDP support for local receiver\n\n---------\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2025-06-12T15:44:12Z",
          "tree_id": "95f83076f82a3cf638d71be5a7518730c2f33f59",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/322e82a2f7edabe612ad372ee0887eb650c13198"
        },
        "date": 1749743646719,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 472000,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 14160000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 14160000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-avg",
            "value": 2.47,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 2.83,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 135.63,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 163.1,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 444000,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 13320000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 13320000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 2.29,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 2.81,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 102.24,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 130.8,
            "unit": "MiB"
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
          "id": "73679e39dc5ae418693c3006b0b66a084485b622",
          "message": "chore(deps): update rust crate criterion to v0.6.0 (#557)\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n|---|---|---|---|\n| [criterion](https://bheisler.github.io/criterion.rs/book/index.html)\n([source](https://redirect.github.com/bheisler/criterion.rs)) |\ndev-dependencies | minor | `0.5` -> `0.6` |\n| [criterion](https://bheisler.github.io/criterion.rs/book/index.html)\n([source](https://redirect.github.com/bheisler/criterion.rs)) |\nworkspace.dependencies | minor | `0.5.1` -> `0.6.0` |\n\n---\n\n### Release Notes\n\n<details>\n<summary>bheisler/criterion.rs (criterion)</summary>\n\n###\n[`v0.6.0`](https://redirect.github.com/bheisler/criterion.rs/blob/HEAD/CHANGELOG.md#060---2025-05-17)\n\n[Compare\nSource](https://redirect.github.com/bheisler/criterion.rs/compare/0.5.1...0.6.0)\n\n##### Changed\n\n-   MSRV bumped to 1.80\n- The `real_blackbox` feature no longer has any impact. Criterion always\nuses `std::hint::black_box()` now.\nUsers of `criterion::black_box()` should switch to\n`std::hint::black_box()`.\n-   `clap` dependency unpinned.\n\n##### Fixed\n\n- gnuplot version is now correctly detected when using certain Windows\nbinaries/configurations that used to fail\n\n##### Added\n\n- Async benchmarking with Tokio may be done via a\n`tokio::runtime::Handle`, not only a `tokio::runtime::Runtime`\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - \"before 8am on Monday\" (UTC),\nAutomerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about these\nupdates again.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0MC40MC4zIiwidXBkYXRlZEluVmVyIjoiNDAuNDguNSIsInRhcmdldEJyYW5jaCI6Im1haW4iLCJsYWJlbHMiOlsiZGVwZW5kZW5jaWVzIl19-->\n\n---------\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: Drew Relmas <drewrelmas@gmail.com>\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2025-06-12T15:58:21Z",
          "tree_id": "942de2f5cd55f9b2b066700c16446726d110c63d",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/73679e39dc5ae418693c3006b0b66a084485b622"
        },
        "date": 1749744570752,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 465833.3333333333,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 13975000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 13975000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-avg",
            "value": 2.52,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 2.85,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 143.47,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 168.85,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 441833.3333333333,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 13255000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 13255000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 2.3,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 2.81,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 109.68,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 139.54,
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
          "id": "be8580e8c6263b0cdf95deae8914401a9730e1ac",
          "message": "feat: add null support to adaptive array builders (#583)\n\nPart of https://github.com/open-telemetry/otel-arrow/issues/534\n\nSupport adding nulls to adaptive array builders.\n\nThis PR also fixes all the \"unused\" clippy warnings. Originally this\nmodule had `#[allow(unused)]` on it to avoid unused code clippy warnings\nuntil this was integrated with code to encode OTAP. It _should_ have had\n`#[allow(dead_code)]`, because `unused` also controls clippy warnings\nfor unused imports and unused results, which we don't want.\n\nAfter the next release of arrow, I'll come back and clean up the\n`append_nulls` method in a few places where we're using loops when we\nshould just be able to call `append_nulls` on the underlying builder.\nSome arrow builders don't support this in the current version, but was\nadded here: https://github.com/apache/arrow-rs/pull/7606\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2025-06-12T16:08:16Z",
          "tree_id": "219d0dc6042b1213ddee40ee86351b74bd1eea17",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/be8580e8c6263b0cdf95deae8914401a9730e1ac"
        },
        "date": 1749745093111,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 466333.3333333333,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 13990000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 13990000,
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
            "value": 2.83,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 138.91,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 164.37,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 440833.3333333333,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 13225000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 13225000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 2.31,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 2.79,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 102.96,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 125.61,
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
          "id": "0ac96ac0421cf9dc3277ba1be311da01a37ee17a",
          "message": "[query-engine] Clippy cleanup and pair down for data & primitives (#586)\n\n## Changes\n\n* Resolve clippy warnings for data & primitives\n* Remove stuff not needed for initial prototyping",
          "timestamp": "2025-06-12T20:20:10Z",
          "tree_id": "45cb2d423be0cb63f8bc8efb82a25afc90f9f125",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/0ac96ac0421cf9dc3277ba1be311da01a37ee17a"
        },
        "date": 1749760200590,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 456500,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 13695000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 13695000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-avg",
            "value": 2.45,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 2.8,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 137.49,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 163.57,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 433666.6666666667,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 13010000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 13010000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 2.29,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 2.78,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 111.66,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 135.73,
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
          "id": "3cf099a0832837429282a85d54275aef63143ea3",
          "message": "[query-engine] Clippy cleanup and pair down for recordset engine root files (#587)\n\n## Changes\n\n* Resolve clippy warnings for recordset engine root files\n* Remove stuff not needed for initial prototyping",
          "timestamp": "2025-06-12T22:08:08Z",
          "tree_id": "032ad9fd15e12d419ff1ca56ce1ca52b982370cd",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/3cf099a0832837429282a85d54275aef63143ea3"
        },
        "date": 1749766690906,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 472500,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 14175000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 14175000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-avg",
            "value": 2.52,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 2.85,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 140.25,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 169.58,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 438500,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 13155000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 13155000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 2.31,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 2.81,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 107.63,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 123,
            "unit": "MiB"
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
          "id": "4d7598536817382d605199093636636bd745cc9c",
          "message": "OTLP Pdata Visitor pattern work-in-progress ready-to-review (#580)\n\nReplaces #507.\nPart of #506.\n\nTested: ✅ Four OTLP encode_len tests have been marked with #[ignore]\nbecause the protobuf logic is incorrect, but it's close so some of the\ntests are passing.\n\nDocmentation: ✅ See src/pdata/otlp/README.md\n\nA new \"item_count\" benchmark is added for counting logs records, as a\nprototype.\n```\nOTLP Logs counting/Visitor\n                        time:   [2.3724 ns 2.4550 ns 2.5344 ns]\nOTLP Logs counting/Manual\n                        time:   [1.0240 ns 1.0703 ns 1.1206 ns]\nOTLP Logs counting/FlatMap\n                        time:   [3.2953 ns 3.4499 ns 3.6075 ns]\n\n```\n\n\n```\nOTLP Logs Serialization/LogsData Prost encode\n                        time:   [751.92 µs 774.70 µs 798.81 µs]\nOTLP Logs Serialization/LogsData Prost encoded_len\n                        time:   [83.360 µs 86.573 µs 90.076 µs]\nOTLP Logs Serialization/LogsData Visitor precompute_sizes\n                        time:   [1.1321 ms 1.1657 ms 1.2030 ms]\nOTLP Logs Serialization/LogsData Prost decode\n                        time:   [2.7549 ms 2.8485 ms 2.9458 ms]\n```\n\nThese results are not super!",
          "timestamp": "2025-06-12T22:26:14Z",
          "tree_id": "a689cfd17b52c031f2c65bd69b65a95f941cb626",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/4d7598536817382d605199093636636bd745cc9c"
        },
        "date": 1749767776133,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 475000,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 14250000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 14250000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-avg",
            "value": 2.49,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 2.83,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 137.54,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 171.04,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 443666.6666666667,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 13310000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 13310000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 2.33,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 2.81,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 115.16,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 141.27,
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
          "id": "cf14576fcd15c03134813a81aed87e93184c2f78",
          "message": "[query-engine] Add basic logical expressions into recordset engine (#588)\n\n## Changes\n\n* Adds basic logical expressions (`==`, `!=`, `>`, `>=`, `<`, `<=`) into\nrecordset engine",
          "timestamp": "2025-06-12T23:04:01Z",
          "tree_id": "09cd3e9650cf64af0d79e3c0f23f253e49b9357f",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/cf14576fcd15c03134813a81aed87e93184c2f78"
        },
        "date": 1749770039968,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 472000,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 14160000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 14160000,
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
            "value": 2.83,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 143.48,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 170.54,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 441333.3333333333,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 13240000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 13240000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 2.37,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 2.8,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 117.5,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 139.52,
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
          "id": "9bb9a9268d1c042f7dff84d90cab969c7813e6e7",
          "message": "[query-engine] Add basic transformation expressions into recordset engine (#590)\n\n## Changes\n\n* Adds basic transformation expressions (set, remove, move, clear) into\nrecordset engine",
          "timestamp": "2025-06-13T19:10:14Z",
          "tree_id": "b8477262257f6158e56006ee1ad4a7c379010f80",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/9bb9a9268d1c042f7dff84d90cab969c7813e6e7"
        },
        "date": 1749842420212,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 471666.6666666667,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 14150000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 14150000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-avg",
            "value": 2.49,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 2.82,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 149.25,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 184.11,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 443166.6666666667,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 13295000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 13295000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 2.4,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 2.8,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 105.37,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 132.32,
            "unit": "MiB"
          }
        ]
      }
    ]
  }
}