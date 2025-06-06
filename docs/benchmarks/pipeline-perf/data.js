window.BENCHMARK_DATA = {
  "lastUpdate": 1749224137156,
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
      }
    ]
  }
}