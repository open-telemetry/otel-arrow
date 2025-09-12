window.BENCHMARK_DATA = {
  "lastUpdate": 1757638239450,
  "repoUrl": "https://github.com/open-telemetry/otel-arrow",
  "entries": {
    "Benchmark": [
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
          "id": "7855d133f9cb5385e8c45e549d8a908a80d7c3c2",
          "message": "Improve support for indentation in tabular expressions (#912)\n\nFixes #911\n\n---------\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2025-08-14T16:32:26Z",
          "tree_id": "288f7e3fe04ee48b35cad0caf7bb1b60455a6ff6",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/7855d133f9cb5385e8c45e549d8a908a80d7c3c2"
        },
        "date": 1755189872572,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 727666.6666666666,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 21830000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 21830000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-avg",
            "value": 5.65,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 6.72,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 161.14,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 192.79,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 734833.3333333334,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 22045000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 22045000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 5.7,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 6.68,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 126.76,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 151.44,
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
          "id": "16ca15caa7c3ce93b9ba11e80425c102a32fa32f",
          "message": "replace `rename_attributes` function with `transform_attributes` w/ delete (#938)\n\nPart of #813 \n\nReplaces the `rename_attributes` function with a new\n`transform_attributes` function that can replace attribute keys and\ndelete attributes by key in a single operation. The motivation by doing\nthese in bulk is to minimize the number of times we may need to\nmaterialize immutable arrow buffers when doing multiple attribute\ntransforms.\n\nThe one outstanding TODO on this is to remove optional columns that,\nbecause some rows were deleted, now contain entirely null or default\nvalues. I can probably do this in a followup\n\n---------\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2025-08-14T17:20:05Z",
          "tree_id": "bad29b251afe44a809dc0314c41144b4a80e339f",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/16ca15caa7c3ce93b9ba11e80425c102a32fa32f"
        },
        "date": 1755192467722,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 754500,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 22635000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 22635000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-avg",
            "value": 5.71,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 6.85,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 157.5,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 191.41,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 743000,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 22290000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 22290000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 5.58,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 6.58,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 124.98,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 146.39,
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
          "id": "ad7b34262b0df82133567705cb90f0ba45b1657d",
          "message": "fix(deps): update rust crate sysinfo to 0.37.0 (#908)\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n|---|---|---|---|\n| [sysinfo](https://redirect.github.com/GuillaumeGomez/sysinfo) |\ndependencies | minor | `0.36.0` -> `0.37.0` |\n\n---\n\n### Release Notes\n\n<details>\n<summary>GuillaumeGomez/sysinfo (sysinfo)</summary>\n\n###\n[`v0.37.0`](https://redirect.github.com/GuillaumeGomez/sysinfo/blob/HEAD/CHANGELOG.md#0370)\n\n[Compare\nSource](https://redirect.github.com/GuillaumeGomez/sysinfo/compare/v0.36.1...v0.37.0)\n\n- Update minimum supported Rust version to `1.88` (for 2024 edition and\n`if let chain` feature).\n- Added `Component::id` API.\n- Linux: Greatly improve partial processes retrieval.\n- Linux: Simplify internal components retrieval code.\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - \"before 8am on Monday\" (UTC),\nAutomerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0MS42MC40IiwidXBkYXRlZEluVmVyIjoiNDEuNjAuNCIsInRhcmdldEJyYW5jaCI6Im1haW4iLCJsYWJlbHMiOlsiZGVwZW5kZW5jaWVzIl19-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2025-08-14T17:27:28Z",
          "tree_id": "549c64c05e884533ca9fe897b825446122575abf",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/ad7b34262b0df82133567705cb90f0ba45b1657d"
        },
        "date": 1755192907436,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 745500,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 22365000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 22365000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-avg",
            "value": 5.87,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 6.82,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 165.48,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 198.7,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 734833.3333333334,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 22045000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 22045000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 5.64,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 6.76,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 123.98,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 145.71,
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
          "id": "d4626562d21bf8e0ad3fdc6c2f9eed8c5fbdaa87",
          "message": "chore(deps): update dependency pyarrow to v21 (#925)\n\nThis PR contains the following updates:\n\n| Package | Change | Age | Confidence |\n|---|---|---|---|\n| [pyarrow](https://redirect.github.com/apache/arrow) | `==20.0.0` ->\n`==21.0.0` |\n[![age](https://developer.mend.io/api/mc/badges/age/pypi/pyarrow/21.0.0?slim=true)](https://docs.renovatebot.com/merge-confidence/)\n|\n[![confidence](https://developer.mend.io/api/mc/badges/confidence/pypi/pyarrow/20.0.0/21.0.0?slim=true)](https://docs.renovatebot.com/merge-confidence/)\n|\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - \"before 8am on Monday\" (UTC),\nAutomerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0MS42MC40IiwidXBkYXRlZEluVmVyIjoiNDEuNjYuMiIsInRhcmdldEJyYW5jaCI6Im1haW4iLCJsYWJlbHMiOlsiZGVwZW5kZW5jaWVzIl19-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2025-08-14T17:34:35Z",
          "tree_id": "bc385f904d978e3114009703e144807996362f44",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/d4626562d21bf8e0ad3fdc6c2f9eed8c5fbdaa87"
        },
        "date": 1755193366993,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 745166.6666666666,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 22355000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 22355000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-avg",
            "value": 5.76,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 6.75,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 162.13,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 191.93,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 737500,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 22125000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 22125000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 5.49,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 6.6,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 128.05,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 145.26,
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
          "id": "4a51d1968e215a429e279dce166241979a779dc2",
          "message": "[query-engine] Diagnostic improvements (#942)\n\n## Changes\n\n* Adds a few missing warning diagnostics for invalid input conditions",
          "timestamp": "2025-08-14T19:44:43Z",
          "tree_id": "4c2d0e51116849a082d9c2762bbc7b96a4eeceeb",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/4a51d1968e215a429e279dce166241979a779dc2"
        },
        "date": 1755201145625,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 752500,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 22575000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 22575000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-avg",
            "value": 5.85,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 6.83,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 165.1,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 192.03,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 747666.6666666666,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 22430000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 22430000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 5.78,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 6.93,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 128.34,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 149.17,
            "unit": "MiB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "990479+MichaelScofield@users.noreply.github.com",
            "name": "LFC",
            "username": "MichaelScofield"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "19aade6bf38b6599574ddc492981465f4468dc83",
          "message": "minor: use \"https\" in the submodule's url (#944)\n\ninstead of \"git@\" to avoid having to set a ssh key to build\notel-arrow-rust. it can be a little annoying for ci",
          "timestamp": "2025-08-15T15:48:48Z",
          "tree_id": "51428ae4166536a698afdc18c553e238e1cf156f",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/19aade6bf38b6599574ddc492981465f4468dc83"
        },
        "date": 1755273547372,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 745833.3333333334,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 22375000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 22375000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-avg",
            "value": 5.75,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 6.81,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 159.02,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 187.64,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 742166.6666666666,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 22265000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 22265000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 5.51,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 6.59,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 122.73,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 142.25,
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
          "id": "ef69d57075457ce2fe518e8b810e0cb9f7f8799e",
          "message": "[query-engine] List expression (#940)\n\n## Changes\n\n* Introduce list expression and use that for KQL `in` operation",
          "timestamp": "2025-08-15T18:43:17Z",
          "tree_id": "40871e7c00b049d59c2330a1a3f10fff4cf489b2",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/ef69d57075457ce2fe518e8b810e0cb9f7f8799e"
        },
        "date": 1755283882271,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 745500,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 22365000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 22365000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-avg",
            "value": 5.71,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 6.68,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 170.7,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 199.61,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 749833.3333333334,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 22495000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 22495000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 5.63,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 6.77,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 124.11,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 144.17,
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
          "id": "8308fcede13549bb3e0a72a1587a810cb27a5623",
          "message": "[query-engine] Remove unused timestamp fields from Record trait (#945)\n\n## Changes\n\n* Remove unused timestamp fields from `Record` trait",
          "timestamp": "2025-08-15T18:54:06Z",
          "tree_id": "d3c28694e73125c2730557041053d8903fce5d8b",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/8308fcede13549bb3e0a72a1587a810cb27a5623"
        },
        "date": 1755284510179,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 748500,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 22455000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 22455000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-avg",
            "value": 5.89,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 6.93,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 161.35,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 196.09,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 744333.3333333334,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 22330000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 22330000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 5.65,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 6.75,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 140.11,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 164.08,
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
          "id": "0f5f8e4d08ea26b5c34ade2c556446e01ddcf092",
          "message": "chore(deps): update golang docker tag to v1.25 (#949)\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n|---|---|---|---|\n| golang | stage | minor | `1.24` -> `1.25` |\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - \"before 8am on Monday\" (UTC),\nAutomerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0MS43MS4xIiwidXBkYXRlZEluVmVyIjoiNDEuNzEuMSIsInRhcmdldEJyYW5jaCI6Im1haW4iLCJsYWJlbHMiOlsiZGVwZW5kZW5jaWVzIl19-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2025-08-18T14:43:50Z",
          "tree_id": "bb1081ee026df9d01eebf9636baeb44bec2b4adf",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/0f5f8e4d08ea26b5c34ade2c556446e01ddcf092"
        },
        "date": 1755528717424,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 743333.3333333334,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 22300000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 22300000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-avg",
            "value": 5.9,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 6.86,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 163.73,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 195.59,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 733333.3333333334,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 22000000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 22000000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 5.63,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 6.75,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 123.7,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 147.91,
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
          "id": "8f116a72439e484a0bc54f0f21e8d71daf6efa2c",
          "message": "[query-engine] Adds parsing of KQL substring expression onto Slice query expression (#955)\n\nRelates to #722\n\n## Changes\n\n* Parse KQL `substring` onto Slice expression",
          "timestamp": "2025-08-18T18:16:30Z",
          "tree_id": "f202a53042485eeac5dc0e953f7828f366e7754b",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/8f116a72439e484a0bc54f0f21e8d71daf6efa2c"
        },
        "date": 1755541468408,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 734000,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 22020000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 22020000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-avg",
            "value": 5.68,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 6.72,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 164.47,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 190.43,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 734333.3333333334,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 22030000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 22030000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 5.71,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 6.81,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 116.98,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 136.63,
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
          "id": "8f8239866f5907f629d17a5d7106a7f9a34c1721",
          "message": "[query-engine] Adds parsing of KQL todatetime expression (#956)\n\nRelates to #722\n\n## Changes\n\n* Parse KQL `todatetime` onto `ConvertScalarExpression::DateTime`\nexpression",
          "timestamp": "2025-08-18T18:30:57Z",
          "tree_id": "066fdaf6a6e0ab18bf3dbf12034cf702d34f9358",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/8f8239866f5907f629d17a5d7106a7f9a34c1721"
        },
        "date": 1755542320856,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 750166.6666666666,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 22505000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 22505000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-avg",
            "value": 5.82,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 7.01,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 164.68,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 193.78,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 730500,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 21915000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 21915000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 5.62,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 6.92,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 119.1,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 137.21,
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
          "id": "3e39da066aef10027084df6861c32c3e98d19b28",
          "message": "[query-engine] Adds support for KQL coalesce expression (#957)\n\n## Changes\n\n* Adds support for parsing KQL `coalesce` onto\n`ScalarExpression::Coalesce`",
          "timestamp": "2025-08-18T19:54:24Z",
          "tree_id": "f69b9b1c4b010a50f26d952888d6bd01531f5b78",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/3e39da066aef10027084df6861c32c3e98d19b28"
        },
        "date": 1755547326269,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 737500,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 22125000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 22125000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-avg",
            "value": 5.8,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 6.84,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 167.62,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 194.45,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 734000,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 22020000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 22020000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 5.65,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 6.66,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 120.29,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 141.39,
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
          "id": "f47b10156fb8ba23ee4e589935f3cb1a52110c49",
          "message": "[query-engine] Add support for KQL parse_json expression (#958)\n\nRelates to #722\n\n## Changes\n\n* Adds parsing of KQL `parse_json` expression onto\n`ParseJsonScalarExpression`",
          "timestamp": "2025-08-18T20:44:21Z",
          "tree_id": "45fb1af4c5b140fa65c9cb38d0a562a0a32c2c17",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/f47b10156fb8ba23ee4e589935f3cb1a52110c49"
        },
        "date": 1755550326142,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 740166.6666666666,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 22205000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 22205000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-avg",
            "value": 5.7,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 6.97,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 152.4,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 183.18,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 727833.3333333334,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 21835000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 21835000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 5.52,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 6.54,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 120.42,
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
          "id": "ff5d7e5f542aa2593402672390fd89de6b5f8cc5",
          "message": "[query-engine] Improve validation in KQL string function parsing (#959)\n\n## Changes\n\n* Adds parameter validation in `strlen` and `replace_string`\n* Adjust the query location to point to the inner expression instead of\nthe root rule in `substring` and `parse_json`",
          "timestamp": "2025-08-18T22:46:13Z",
          "tree_id": "ffbf062c70e1b8d1c2fb5c3058ae3ad0623c6cd0",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/ff5d7e5f542aa2593402672390fd89de6b5f8cc5"
        },
        "date": 1755558643067,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 745333.3333333334,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 22360000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 22360000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-avg",
            "value": 5.77,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 6.81,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 165.38,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 193.62,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 733500,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 22005000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 22005000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 5.69,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 6.85,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 124.35,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 144.12,
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
          "id": "7188fc410e027c0b836b1a0925786b3fe52ba57b",
          "message": "[otap-dataflow] dockerfile and supporting for df_engine (#960)\n\nFixes #953 \n\nAdds a dockerfile for otap-dataflow.\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2025-08-19T16:06:52Z",
          "tree_id": "5241bbea511bc4fea8db79e5528c0f09ec73d222",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/7188fc410e027c0b836b1a0925786b3fe52ba57b"
        },
        "date": 1755620181015,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 736166.6666666666,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 22085000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 22085000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-avg",
            "value": 5.77,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 6.77,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 159.32,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 190.2,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 737833.3333333334,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 22135000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 22135000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 5.59,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 6.81,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 131.85,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 154.08,
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
          "id": "c73c96a4fad6d4877ffb37f900c40c0dd784b8c6",
          "message": "chore: Nit fixes to perf test suite (#962)\n\nnit fixes so the readme.md instructions are runnable as-is.",
          "timestamp": "2025-08-19T17:39:20Z",
          "tree_id": "700ec0dc014f03bba426b828fd5379a361c1e3bb",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/c73c96a4fad6d4877ffb37f900c40c0dd784b8c6"
        },
        "date": 1755625630939,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 741666.6666666666,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 22250000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 22250000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-avg",
            "value": 5.67,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 6.79,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 157.91,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 187.77,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 739833.3333333334,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 22195000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 22195000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 5.71,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 6.79,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 129.91,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 150.92,
            "unit": "MiB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "129437996+c1ly@users.noreply.github.com",
            "name": "c1ly",
            "username": "c1ly"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "d4879f499c1819a268de6bb399c842db626b7a4d",
          "message": "[otap-dataflow] Update the fake-signal-receiver to accept a message rate and update attribute key/value pairing to be random (#883)\n\nUpdated the config adding traffic config that will define the message\nper second rate, and the load size of each signal being sent.\nUpdated receiver to take into account the total number of messages from\nthe defined signal load and calculate the interval between each\ngeneration call\n\n---------\n\nCo-authored-by: albertlockett <a.lockett@f5.com>\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2025-08-19T19:53:23Z",
          "tree_id": "377312478255799184219e4f8b7ec9760e9240cd",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/d4879f499c1819a268de6bb399c842db626b7a4d"
        },
        "date": 1755633711010,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 733833.3333333334,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 22015000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 22015000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-avg",
            "value": 5.88,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 7.26,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 159.6,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 188.64,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 734000,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 22020000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 22020000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 5.62,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 6.65,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 127.91,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 156.25,
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
          "id": "d45e70a8368bdc89e9c7c5ee87fc7c1e55ff59cb",
          "message": "Establish engine::NodeId = {config::NodeId; NodeIndex(u16)} (#939)\n\nEnable fast lookup of sender/receiver nodes for delivering Ack/Nack from\nthe engine pipeline controller. `engine::NodeId` is a new struct\ncombining `config::NodeId`, which it reexports as `engine::NodeName` and\n`NodeIndex(u16)`.\n\nThe `engine::NodeId` field is passed through the component interface, so\nthat nodes know how to identify themselves using their unique\nidentifier, which serves as a direct index to a vector of node\ndefinitions. Node definitions also contain an extra field (`inner`)\ncontaining their offset in the appropriate vector (exporter, processor,\nreceiver) vector for direct access instead of a HashMap approach.\n\nNo functional changes. New type names in `engine` crate,\n\n- `NodeIndex`\n- `NodeDefinition`\n- `NodeDefs`\n- `NodeName`\n\nNo changes in `config` crate: T.B.D. a future change can make\nconfig::NodeId into `NodeName`, replacing `engine::NodeName`.\n\nNew test helpers in `crate::engine::testing::{test_node, test_nodes}`.\n\nThe engine::error crate still uses `NodeName` in `UnknownError` where\nthe full `NodeId` is not defined. Adds a `TooManyNodes` error for when\nthe u16 overflows.\n\nPoints of confusion in the review thread:\n- ~See questions about ambiguous dispatch for `send_control_msg`. I\nbelieve I've eliminated an unnecessary code path, but could be wrong!~\n- ~See a question about config validation for the batch processor test,\nwhich was constructing an error incorrectly.~\n\nPart of\n\n#509 \n#919",
          "timestamp": "2025-08-19T23:09:33Z",
          "tree_id": "26cf51d908e3a64de804d5768c0ccc7f2d1d8591",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/d45e70a8368bdc89e9c7c5ee87fc7c1e55ff59cb"
        },
        "date": 1755646092167,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 741833.3333333334,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 22255000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 22255000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-avg",
            "value": 5.75,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 6.82,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 156.56,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 180.21,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 737000,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 22110000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 22110000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 5.65,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 6.68,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 128.12,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 153.58,
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
          "id": "69a56adf9467bd3aa666e30ebd81a5c3a8644b9b",
          "message": "chore(deps): update dependency flask to v3.1.2 (#971)\n\nThis PR contains the following updates:\n\n| Package | Change | Age | Confidence |\n|---|---|---|---|\n| [Flask](https://redirect.github.com/pallets/flask)\n([changelog](https://flask.palletsprojects.com/page/changes/)) |\n`==3.1.1` -> `==3.1.2` |\n[![age](https://developer.mend.io/api/mc/badges/age/pypi/flask/3.1.2?slim=true)](https://docs.renovatebot.com/merge-confidence/)\n|\n[![confidence](https://developer.mend.io/api/mc/badges/confidence/pypi/flask/3.1.1/3.1.2?slim=true)](https://docs.renovatebot.com/merge-confidence/)\n|\n\n---\n\n### Release Notes\n\n<details>\n<summary>pallets/flask (Flask)</summary>\n\n###\n[`v3.1.2`](https://redirect.github.com/pallets/flask/blob/HEAD/CHANGES.rst#Version-312)\n\n[Compare\nSource](https://redirect.github.com/pallets/flask/compare/3.1.1...3.1.2)\n\nReleased 2025-08-19\n\n- `stream_with_context` does not fail inside async views. :issue:`5774`\n- When using `follow_redirects` in the test client, the final state\n  of `session` is correct. :issue:`5786`\n- Relax type hint for passing bytes IO to `send_file`. :issue:`5776`\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - \"before 8am every weekday\" (UTC),\nAutomerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0MS44MS4yIiwidXBkYXRlZEluVmVyIjoiNDEuODEuMiIsInRhcmdldEJyYW5jaCI6Im1haW4iLCJsYWJlbHMiOlsiZGVwZW5kZW5jaWVzIl19-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2025-08-20T14:53:18Z",
          "tree_id": "c50e75702927d6e64fccb556b5314f278947c69c",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/69a56adf9467bd3aa666e30ebd81a5c3a8644b9b"
        },
        "date": 1755702134859,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 736833.3333333334,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 22105000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 22105000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-avg",
            "value": 5.69,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 6.81,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 163.45,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 191.31,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 739833.3333333334,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 22195000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 22195000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 5.72,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 6.76,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 133.83,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 158.49,
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
          "id": "c780e427363c74b70430ed0d8e80fdae43f5ea0a",
          "message": "[query-engine] TimeSpan support in expressions and recordset engine (#970)\n\n## Changes\n\n* Adds support for TimeSpan values in expressions and recordset engine",
          "timestamp": "2025-08-20T17:54:35Z",
          "tree_id": "8cd3e530e13ece5d165b89e19653519cf901d5e5",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/c780e427363c74b70430ed0d8e80fdae43f5ea0a"
        },
        "date": 1755713011137,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 735500,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 22065000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 22065000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-avg",
            "value": 5.8,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 7.05,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 161.91,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 191.85,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 727000,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 21810000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 21810000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 5.64,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 6.67,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 124.95,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 146.11,
            "unit": "MiB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "geukhanuslu@gmail.com",
            "name": "Gokhan Uslu",
            "username": "gouslu"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "9ff1f4e23cd14b5d5f791e92f0282a568a9a04f5",
          "message": "add arithmetic operations query engine support and KQL implementation (#963)\n\naddressing arithmetic operations KQL grammar support on issue #722 \n\ndue to pest parser not supporting circular references, I need to add the\nparenthesis workaround for arithmetic operations. I will look into how\nto fix that using pratt parser together with all the other circular\nreference issues in the scalar_expressions list in the pest file.\n\n---------\n\nCo-authored-by: Mikel Blanchard <mblanchard@macrosssoftware.com>",
          "timestamp": "2025-08-20T17:58:58Z",
          "tree_id": "6c2415830357162cfa15688cf75520987f16bf29",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/9ff1f4e23cd14b5d5f791e92f0282a568a9a04f5"
        },
        "date": 1755713227630,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 732833.3333333334,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 21985000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 21985000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-avg",
            "value": 5.68,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 6.84,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 157.61,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 188.29,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 731333.3333333334,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 21940000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 21940000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 5.77,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 6.8,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 128.44,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 147.33,
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
          "id": "1252211c060c29e186946afa2bdb18d9cbf05718",
          "message": "[query_engine] Timespan, bin, & now support in KQL parser (#974)\n\nRelates to #722\n\n## Changes\n\n* Adds `totimespan(...)`, `timespan(...)`, and time literal (eg `1 h`)\nsupport in the KQL parser\n* Adds `bin` support in the KQL parser\n* Adds `now` support in the KQL parser",
          "timestamp": "2025-08-21T00:33:56Z",
          "tree_id": "bac7baca1bd28f38354426f98e0eae493331c0c9",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/1252211c060c29e186946afa2bdb18d9cbf05718"
        },
        "date": 1755736933953,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 737500,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 22125000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 22125000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-avg",
            "value": 5.77,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 6.87,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 161.84,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 190.75,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 737166.6666666666,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 22115000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 22115000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 5.64,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 6.93,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 116.89,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 140.26,
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
          "id": "b4bb1d87434528b7ffedc6aa5c76b512f6358182",
          "message": "[query-engine] Add a summarize integration test using bin (#979)\n\nnt",
          "timestamp": "2025-08-21T17:51:56Z",
          "tree_id": "863e4a2177b79422ef9699748e07729ec1f69b6c",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/b4bb1d87434528b7ffedc6aa5c76b512f6358182"
        },
        "date": 1755799186501,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 738000,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 22140000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 22140000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-avg",
            "value": 5.77,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 6.82,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 162.08,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 192.86,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 738166.6666666666,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 22145000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 22145000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 5.58,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 6.58,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 120.84,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 142.02,
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
          "id": "154fe91062790c88803a5e303d3bcc06bb5f39c0",
          "message": "[query-engine] Implement missing static resolution for logical expressions (#981)\n\n## Changes\n\n* Implements missing `try_resolve_static` methods for logical\nexpressions",
          "timestamp": "2025-08-21T19:59:53Z",
          "tree_id": "23e4b792144a8a499460b4031c62a16d9b5ced9d",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/154fe91062790c88803a5e303d3bcc06bb5f39c0"
        },
        "date": 1755806884366,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 737500,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 22125000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 22125000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-avg",
            "value": 5.81,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 6.89,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 160.17,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 192.26,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 735500,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 22065000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 22065000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 5.64,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 6.59,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 123.69,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 145.04,
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
          "id": "369e081ed3d57a0fd925b85bcabcc052a29436c4",
          "message": "[query-engine] Global variable support (#982)\n\n## Changes\n\n* Expands the pipeline so it can track global variables scoped to a\nbatch of records\n\n## Details\n\nThe main scenario I wanted to enable is this:\n\n```\nlet batch_time = now();\nsource | summarize by BatchTime = batch_time\n```\n\nBut also planning to utilize global variables elsewhere. Looking ahead\nto user-defined functions:\n\n```\nlet c = 10; // global variable\nlet Func = (v:long) { \n    let c = c + 1; // creates a local variable with the same name as the global initialized to the global value + 1\n    v + c\n};\nprint Func(0), Func(1)\n// output: 11, 12\n```\n\nPerhaps also something like:\n\n```\nlet lookup = external_call_to_get_data();\nsource | extend name = lookup[id]\n```",
          "timestamp": "2025-08-21T21:07:11Z",
          "tree_id": "0b068faf316f95b8ef9e1fd4c593fc62f628d776",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/369e081ed3d57a0fd925b85bcabcc052a29436c4"
        },
        "date": 1755810921836,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 749166.6666666666,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 22475000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 22475000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-avg",
            "value": 5.73,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 6.75,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 166.31,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 196.39,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 735500,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 22065000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 22065000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 5.56,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 6.68,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 120.7,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 138.51,
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
          "id": "bbf9a82b8b835ba4012aa5e81969cadaa2a3e30d",
          "message": "fix: golang OTAP decoding logs handles optional trace ID and span ID columns (#973)\n\nFixes: #951",
          "timestamp": "2025-08-21T21:24:40Z",
          "tree_id": "109e8027ac255d0f00459aedc932b953ff5ff70c",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/bbf9a82b8b835ba4012aa5e81969cadaa2a3e30d"
        },
        "date": 1755811971755,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 741000,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 22230000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 22230000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-avg",
            "value": 5.82,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 6.91,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 162.1,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 191.8,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 725333.3333333334,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 21760000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 21760000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 5.46,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 6.57,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 125.91,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 147.25,
            "unit": "MiB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "129437996+c1ly@users.noreply.github.com",
            "name": "c1ly",
            "username": "c1ly"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "4e3c47dc78d35876b565c4db11ee8b618a9e330d",
          "message": "[otap-dataflow] add max batch setting to fake signal generator (#976)\n\nAdd a max batch setting to cap the number of signals being sent per\nsecond.\nInstead of sending one message with x amount of signals if x is greater\nthan the max batch limit of y then we break up x so the signals are sent\nin multiple messages.\n\n---------\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2025-08-21T22:01:43Z",
          "tree_id": "3f95c6631270dc9e1dec649e32923f5a3b669c9f",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/4e3c47dc78d35876b565c4db11ee8b618a9e330d"
        },
        "date": 1755814203699,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 745166.6666666666,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 22355000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 22355000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-avg",
            "value": 5.76,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 6.69,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 157.22,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 183.92,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 741500,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 22245000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 22245000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 5.63,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 6.58,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 118.71,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 137.53,
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
          "id": "49488a8084ca96bc92e8247f336eb454d1739ab0",
          "message": "Add benchmarks for Syslog CEF Receiver (#978)\n\n## Changes\n- Add benchmarks for Syslog CEF Receiver\n\nMachine details:\n\nOS: Ubuntu 22.04.4 LTS (5.15.153.1-microsoft-standard-WSL2)\nHardware: Intel(R) Xeon(R) Platinum 8370C CPU @ 2.80GHz, 16vCPUs,\nRAM: 64.0 GB\n\n**parser_comparison/rfc3164**\n```\ntime:   [22.645 ns 22.859 ns 23.178 ns]\nthrpt:  [3.0538 GiB/s 3.0965 GiB/s 3.1257 GiB/s]\n```\n\n**parser_comparison/rfc5424**\n```\ntime:   [60.350 ns 60.541 ns 60.803 ns]\nthrpt:  [1.1028 GiB/s 1.1076 GiB/s 1.1111 GiB/s]\n```\n\n**parser_comparison/cef**\n```\ntime:   [43.112 ns 43.256 ns 43.452 ns]\nthrpt:  [1.6718 GiB/s 1.6794 GiB/s 1.6850 GiB/s]\n```\n\n**arrow_batch_creation/rfc3164_arrow_batch_100_msgs**\n```\ntime:   [149.85 µs 150.54 µs 151.27 µs]\n```\n\n**arrow_batch_creation/rfc5424_arrow_batch_100_msgs**\n```\ntime:   [126.32 µs 128.02 µs 130.03 µs]\n```\n\n**arrow_batch_creation/cef_arrow_batch_100_msgs**\n```\ntime:   [94.159 µs 94.257 µs 94.372 µs]\n```\n\n---------\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2025-08-21T22:57:20Z",
          "tree_id": "77446ebfdb44e370719e2bebefa860bfb978d022",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/49488a8084ca96bc92e8247f336eb454d1739ab0"
        },
        "date": 1755817807178,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 738333.3333333334,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 22150000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 22150000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-avg",
            "value": 5.78,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 7.06,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 164.49,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 202.56,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 739333.3333333334,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 22180000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 22180000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 5.6,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 6.7,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 128.07,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 150.06,
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
          "id": "a679d7eba9f4e53cc0f67ee472fb3d7779c8dd1a",
          "message": "[otap-dataflow ] Add OTAP attributes_processor  (#941)\n\nAdds a new OTAP processor that enables efficient bulk renaming of\nattributes across logs, metrics, and traces. The processor:\n\n- Uses the transform_attributes API for safe, atomic attribute\ntransformations\n- Works on both OtapArrowRecords and OtapArrowBytes payloads\n- Precomputes rename maps for improved performance\n- Prevents ambiguous renames (e.g., A->B->C) by validating transforms\n- Includes comprehensive tests across all signal types\n\nAdditional changes:\n- Updated string format patterns in perf_exporter and otap_exporter to\ncomply with new Clippy requirements around string interpolation.\n\n---------\n\nCo-authored-by: albertlockett <a.lockett@f5.com>\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2025-08-21T23:08:56Z",
          "tree_id": "d2ffed80b88615381ede57730f9e3418cd93d552",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/a679d7eba9f4e53cc0f67ee472fb3d7779c8dd1a"
        },
        "date": 1755818220747,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 734000,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 22020000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 22020000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-avg",
            "value": 5.61,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 6.62,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 160.7,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 188.1,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 733833.3333333334,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 22015000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 22015000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 5.7,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 6.85,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 129.22,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 151.36,
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
          "id": "fc5780ec74a02a55f295fad2ef5550b65085fa55",
          "message": "Update Node index to usize (#984)\n\nFollow up to\nhttps://github.com/open-telemetry/otel-arrow/pull/939#discussion_r2286670031\n\n## Changes\n- Update Node index to `usize` to look up the vec directly without\nconverting from `u16`",
          "timestamp": "2025-08-21T23:12:26Z",
          "tree_id": "10812c04ed420619d3564c10bca1a082c560875c",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/fc5780ec74a02a55f295fad2ef5550b65085fa55"
        },
        "date": 1755818458650,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 737666.6666666666,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 22130000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 22130000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-avg",
            "value": 5.67,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 6.79,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 168.5,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 194.96,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 736666.6666666666,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 22100000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 22100000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 5.73,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 6.69,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 133.84,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 154.46,
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
          "id": "7b99f046feba2a73a64fe0356425de01023b7c36",
          "message": "[otap-controller] Update Result Type (#987)\n\n## Changes\n- Update the result type of `run_pipeline_thread` and `run_forever`\nmethod",
          "timestamp": "2025-08-22T00:01:32Z",
          "tree_id": "7dc70d9fa947afe24d375b6baa5e8781a9f4b97d",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/7b99f046feba2a73a64fe0356425de01023b7c36"
        },
        "date": 1755821354714,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 742666.6666666666,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 22280000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 22280000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-avg",
            "value": 5.79,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 6.8,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 164.55,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 192.76,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 734166.6666666666,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 22025000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 22025000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 5.66,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 6.56,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 132.08,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 155.86,
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
          "distinct": false,
          "id": "99f37122bed26cb29e8b567e043dd293df7ee148",
          "message": "[otap-controller] Fix error message (#988)\n\n## Changes\n- Use `coreId` instead of enumeration index in the error message since\nwe use `coreId` when naming the thread\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2025-08-22T15:18:04Z",
          "tree_id": "7aaccde0eb5fd742f7a16defa806f0b492eb6b74",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/99f37122bed26cb29e8b567e043dd293df7ee148"
        },
        "date": 1755876502802,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 733500,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 22005000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 22005000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-avg",
            "value": 5.64,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 6.68,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 158.47,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 182.2,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 734500,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 22035000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 22035000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 5.5,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 6.7,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 117.46,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 138.65,
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
          "id": "4a36e25ec8b04424c6ee14c70cde21723300675b",
          "message": "Restore parquet exporter shutdown test and fix flakey implementation (#991)\n\nFixes: #967 \n\nI think the source of the flakiness here was we'd enter into this:\n\nhttps://github.com/open-telemetry/otel-arrow/blob/b4bb1d87434528b7ffedc6aa5c76b512f6358182/rust/otap-dataflow/crates/engine/src/message.rs#L221-L241\n\nAnd depending on how long it took between when the test setup function\nrun, and when we invoked the validation function, we'd either hit the\ndeadline or the pdata first. If we hit the deadline first, no writes are\nqueued and the parquet exporter shuts down successfuly. This is not what\nwe're trying to test here.\n\nTo get around this, the test is now rewritten to manually drive the\ncontrol messages, and have the parquet exporter receiving them at the\nsame time. This way, we can have a better guarantee that there will be\nunflushed writes queued when it receives the shutdown signal.",
          "timestamp": "2025-08-22T16:19:51Z",
          "tree_id": "c94187dcd1e193bdbea35c999355c15c948d654c",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/4a36e25ec8b04424c6ee14c70cde21723300675b"
        },
        "date": 1755880090367,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 737500,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 22125000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 22125000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-avg",
            "value": 5.72,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 6.8,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 162.45,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 188.84,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 743833.3333333334,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 22315000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 22315000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 5.7,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 6.77,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 128.31,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 150.53,
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
          "id": "f072056cceec81bda170e322411f0ed1d28b7720",
          "message": "fix resource and scope IDs not getting decoded when removing transport encoding (#993)\n\nFixes: #952 \n\nWhen we remove the transport encoding from an OTAP Batch (e.g. removing\ndelta encoding ID columns so they're plain encoded), we were missing the\nresource/scope ID columns embedded within these struct arrays on the\nroot record. This fixes the bug.",
          "timestamp": "2025-08-22T20:13:58Z",
          "tree_id": "3325736d78a693ac853275ec9a01f64cd8e22152",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/f072056cceec81bda170e322411f0ed1d28b7720"
        },
        "date": 1755894142893,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 730833.3333333334,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 21925000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 21925000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-avg",
            "value": 5.75,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 6.87,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 156.49,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 186.98,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 737333.3333333334,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 22120000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 22120000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 5.6,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 6.59,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 123.87,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 146.18,
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
          "id": "88fabbd4e2b5980d66e9ef459cbd0fe9d4a285d3",
          "message": "[otap-df-engine] Simplify timer control (#996)\n\n## Changes\n- Use only one HashMap to track the timer state instead of three\ndifferent ones\n- Instead of 3-4 HashMap lookups per operation we now only have one\nlookup\n- This also offers better cache locality",
          "timestamp": "2025-08-22T21:51:26Z",
          "tree_id": "560bff9797c2a7d4dda3ddfb16e6c859dd7fd0f5",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/88fabbd4e2b5980d66e9ef459cbd0fe9d4a285d3"
        },
        "date": 1755899999091,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 738666.6666666666,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 22160000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 22160000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-avg",
            "value": 5.88,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 6.84,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 170.81,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 199.14,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 741000,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 22230000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 22230000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 5.71,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 6.7,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 129.33,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 153.85,
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
          "id": "9535da4ec518f115098f7c5300975b5827b58510",
          "message": "PerfTest - add syslog loadgen (#990)\n\nFor now, just modified the existing comparison between OTelCollector\nbatch vs non-batching to use syslog instead of OTLP.",
          "timestamp": "2025-08-22T22:16:06Z",
          "tree_id": "96333094a8ccff0d588bd227f87cc4c4e1ca7376",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/9535da4ec518f115098f7c5300975b5827b58510"
        },
        "date": 1755901482318,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 733833.3333333334,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 22015000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 22015000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-avg",
            "value": 5.7,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 6.71,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 168.59,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 210.5,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 737333.3333333334,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 22120000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 22120000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 5.53,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 6.73,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 128.69,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 150.94,
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
          "id": "d9beabf086507cd161ff9d36273893fa52805519",
          "message": "[query-engine] Add matches logical expression and implementation (#997)\n\n## Changes\n\n* Adds an expression and implementation for performing regex string\nmatch evaluation",
          "timestamp": "2025-08-22T23:07:47Z",
          "tree_id": "59b06e90d9458942819cf554f95166dbadbed6d0",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/d9beabf086507cd161ff9d36273893fa52805519"
        },
        "date": 1755904560460,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 741000,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 22230000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 22230000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-avg",
            "value": 5.78,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 6.92,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 155.18,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 184.02,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 739833.3333333334,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 22195000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 22195000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 5.65,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 6.88,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 138.18,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 160.4,
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
          "id": "132ecad3a6f8a0eae8ff50ce0154cde146fe037e",
          "message": "[query-engine] Support expressions run against summary records (#995)\n\n## Changes\n\n* Adds support for running expressions over the summary records once\nthey are generated",
          "timestamp": "2025-08-22T23:22:55Z",
          "tree_id": "63e2274afb17aed04de2cf0c60ffa56fd1642035",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/132ecad3a6f8a0eae8ff50ce0154cde146fe037e"
        },
        "date": 1755905483768,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 736500,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 22095000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 22095000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-avg",
            "value": 5.74,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 6.85,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 176.09,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 205.57,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 729666.6666666666,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 21890000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 21890000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 5.63,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 6.52,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 132.92,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 154.15,
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
          "id": "b0f3afb669bb7c2e877953ac984ae4d1817d340b",
          "message": "Update Rust crate arrow to v56 (#1002)\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n|---|---|---|---|\n| [arrow](https://redirect.github.com/apache/arrow-rs) |\nworkspace.dependencies | major | `55` -> `56` |\n\n---\n\n### Release Notes\n\n<details>\n<summary>apache/arrow-rs (arrow)</summary>\n\n###\n[`v56.1.0`](https://redirect.github.com/apache/arrow-rs/blob/HEAD/CHANGELOG.md#5610-2025-08-21)\n\n[Compare\nSource](https://redirect.github.com/apache/arrow-rs/compare/56.0.0...56.1.0)\n\n[Full\nChangelog](https://redirect.github.com/apache/arrow-rs/compare/56.0.0...56.1.0)\n\n**Implemented enhancements:**\n\n- Implement cast and other operations on decimal32 and decimal64\n[#&#8203;7815](https://redirect.github.com/apache/arrow-rs/issues/7815)\n[#&#8203;8204](https://redirect.github.com/apache/arrow-rs/issues/8204)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n- Speed up Parquet filter pushdown with predicate cache\n[#&#8203;8203](https://redirect.github.com/apache/arrow-rs/issues/8203)\n\\[[parquet](https://redirect.github.com/apache/arrow-rs/labels/parquet)]\n- Optionally read parquet page indexes\n[#&#8203;8070](https://redirect.github.com/apache/arrow-rs/issues/8070)\n\\[[parquet](https://redirect.github.com/apache/arrow-rs/labels/parquet)]\n- Parquet reader: add method for sync reader read bloom filter\n[#&#8203;8023](https://redirect.github.com/apache/arrow-rs/issues/8023)\n\\[[parquet](https://redirect.github.com/apache/arrow-rs/labels/parquet)]\n- \\[parquet] Support writing logically equivalent types to `ArrowWriter`\n[#&#8203;8012](https://redirect.github.com/apache/arrow-rs/issues/8012)\n\\[[parquet](https://redirect.github.com/apache/arrow-rs/labels/parquet)]\n- Improve StringArray(Utf8) sort performance\n[#&#8203;7847](https://redirect.github.com/apache/arrow-rs/issues/7847)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n- feat: arrow-ipc delta dictionary support\n[#&#8203;8001](https://redirect.github.com/apache/arrow-rs/pull/8001)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n([JakeDern](https://redirect.github.com/JakeDern))\n\n**Fixed bugs:**\n\n- The Rustdocs are clean CI job is failing\n[#&#8203;8175](https://redirect.github.com/apache/arrow-rs/issues/8175)\n- \\[avro] Bug in resolving avro schema with named type\n[#&#8203;8045](https://redirect.github.com/apache/arrow-rs/issues/8045)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n- Doc test failure (test arrow-avro/src/lib.rs - reader) when verifying\navro 56.0.0 RC1 release\n[#&#8203;8018](https://redirect.github.com/apache/arrow-rs/issues/8018)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n\n**Documentation updates:**\n\n- arrow-row: Document dictionary handling\n[#&#8203;8168](https://redirect.github.com/apache/arrow-rs/pull/8168)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n([alamb](https://redirect.github.com/alamb))\n- Docs: Clarify that Array::value does not check for nulls\n[#&#8203;8065](https://redirect.github.com/apache/arrow-rs/pull/8065)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n([alamb](https://redirect.github.com/alamb))\n- docs: Fix a typo in README\n[#&#8203;8036](https://redirect.github.com/apache/arrow-rs/pull/8036)\n([EricccTaiwan](https://redirect.github.com/EricccTaiwan))\n- Add more comments to the internal parquet reader\n[#&#8203;7932](https://redirect.github.com/apache/arrow-rs/pull/7932)\n\\[[parquet](https://redirect.github.com/apache/arrow-rs/labels/parquet)]\n([alamb](https://redirect.github.com/alamb))\n\n**Performance improvements:**\n\n- perf(arrow-ipc): avoid counting nulls in `RecordBatchDecoder`\n[#&#8203;8127](https://redirect.github.com/apache/arrow-rs/pull/8127)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n([rluvaton](https://redirect.github.com/rluvaton))\n- Use `Vec` directly in builders\n[#&#8203;7984](https://redirect.github.com/apache/arrow-rs/pull/7984)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n([liamzwbao](https://redirect.github.com/liamzwbao))\n- Improve StringArray(Utf8) sort performance (\\~2-4x faster)\n[#&#8203;7860](https://redirect.github.com/apache/arrow-rs/pull/7860)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n([zhuqi-lucas](https://redirect.github.com/zhuqi-lucas))\n\n**Closed issues:**\n\n- \\[Variant] Improve fuzz test for Variant\n[#&#8203;8199](https://redirect.github.com/apache/arrow-rs/issues/8199)\n- \\[Variant] Improve fuzz test for Variant\n[#&#8203;8198](https://redirect.github.com/apache/arrow-rs/issues/8198)\n- `VariantArrayBuilder` tracks starting offsets instead of (offset, len)\npairs\n[#&#8203;8192](https://redirect.github.com/apache/arrow-rs/issues/8192)\n- Rework `ValueBuilder` API to work with `ParentState` for reliable\nnested rollbacks\n[#&#8203;8188](https://redirect.github.com/apache/arrow-rs/issues/8188)\n- \\[Variant] Rename `ValueBuffer` as `ValueBuilder`\n[#&#8203;8186](https://redirect.github.com/apache/arrow-rs/issues/8186)\n- \\[Variant] Refactor `ParentState` to track and rollback state on\nbehalf of its owning builder\n[#&#8203;8182](https://redirect.github.com/apache/arrow-rs/issues/8182)\n- \\[Variant] `ObjectBuilder` should detect duplicates at insertion time,\nnot at finish\n[#&#8203;8180](https://redirect.github.com/apache/arrow-rs/issues/8180)\n- \\[Variant] ObjectBuilder does not reliably check for duplicates\n[#&#8203;8170](https://redirect.github.com/apache/arrow-rs/issues/8170)\n- \\[Variant] Support `StringView` and `LargeString` in\n´batch\\_json\\_string\\_to\\_variant\\`\n[#&#8203;8145](https://redirect.github.com/apache/arrow-rs/issues/8145)\n\\[[parquet](https://redirect.github.com/apache/arrow-rs/labels/parquet)]\n- \\[Variant] Rename `batch_json_string_to_variant` and\n`batch_variant_to_json_string` json\\_to\\_variant\n[#&#8203;8144](https://redirect.github.com/apache/arrow-rs/issues/8144)\n\\[[parquet](https://redirect.github.com/apache/arrow-rs/labels/parquet)]\n- \\[avro] Use `tempfile` crate rather than custom temporary file\ngenerator in tests\n[#&#8203;8143](https://redirect.github.com/apache/arrow-rs/issues/8143)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n- \\[Avro] Use `Write` rather `dyn Write` in Decoder\n[#&#8203;8142](https://redirect.github.com/apache/arrow-rs/issues/8142)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n- \\[Variant] Nested builder rollback is broken\n[#&#8203;8136](https://redirect.github.com/apache/arrow-rs/issues/8136)\n- \\[Variant] Add support the remaing primitive\ntype(timestamp\\_nanos/timestampntz\\_nanos/uuid) for parquet variant\n[#&#8203;8126](https://redirect.github.com/apache/arrow-rs/issues/8126)\n- Meta: Implement missing Arrow 56.0 lint rules - Sequential workflow\n[#&#8203;8121](https://redirect.github.com/apache/arrow-rs/issues/8121)\n- ARROW-012-015: Add linter rules for remaining Arrow 56.0 breaking\nchanges\n[#&#8203;8120](https://redirect.github.com/apache/arrow-rs/issues/8120)\n- ARROW-010 & ARROW-011: Add linter rules for Parquet Statistics and\nMetadata API removals\n[#&#8203;8119](https://redirect.github.com/apache/arrow-rs/issues/8119)\n- ARROW-009: Add linter rules for IPC Dictionary API removals in Arrow\n56.0\n[#&#8203;8118](https://redirect.github.com/apache/arrow-rs/issues/8118)\n- ARROW-008: Add linter rule for SerializedPageReaderState usize→u64\nbreaking change\n[#&#8203;8117](https://redirect.github.com/apache/arrow-rs/issues/8117)\n- ARROW-007: Add linter rule for Schema.all\\_fields() removal in Arrow\n56.0\n[#&#8203;8116](https://redirect.github.com/apache/arrow-rs/issues/8116)\n- \\[Variant] Implement `ShreddingState::AllNull` variant\n[#&#8203;8088](https://redirect.github.com/apache/arrow-rs/issues/8088)\n\\[[parquet](https://redirect.github.com/apache/arrow-rs/labels/parquet)]\n- \\[Variant] Support Shredded Objects in `variant_get`\n[#&#8203;8083](https://redirect.github.com/apache/arrow-rs/issues/8083)\n\\[[parquet](https://redirect.github.com/apache/arrow-rs/labels/parquet)]\n- \\[Variant]: Implement `DataType::RunEndEncoded` support for\n`cast_to_variant` kernel\n[#&#8203;8064](https://redirect.github.com/apache/arrow-rs/issues/8064)\n\\[[parquet](https://redirect.github.com/apache/arrow-rs/labels/parquet)]\n- \\[Variant]: Implement `DataType::Dictionary` support for\n`cast_to_variant` kernel\n[#&#8203;8062](https://redirect.github.com/apache/arrow-rs/issues/8062)\n\\[[parquet](https://redirect.github.com/apache/arrow-rs/labels/parquet)]\n- \\[Variant]: Implement `DataType::Struct` support for `cast_to_variant`\nkernel\n[#&#8203;8061](https://redirect.github.com/apache/arrow-rs/issues/8061)\n\\[[parquet](https://redirect.github.com/apache/arrow-rs/labels/parquet)]\n- \\[Variant]: Implement\n`DataType::Decimal32/Decimal64/Decimal128/Decimal256` support for\n`cast_to_variant` kernel\n[#&#8203;8059](https://redirect.github.com/apache/arrow-rs/issues/8059)\n\\[[parquet](https://redirect.github.com/apache/arrow-rs/labels/parquet)]\n- \\[Variant]: Implement `DataType::Timestamp(..)` support for\n`cast_to_variant` kernel\n[#&#8203;8058](https://redirect.github.com/apache/arrow-rs/issues/8058)\n\\[[parquet](https://redirect.github.com/apache/arrow-rs/labels/parquet)]\n- \\[Variant]: Implement `DataType::Float16` support for\n`cast_to_variant` kernel\n[#&#8203;8057](https://redirect.github.com/apache/arrow-rs/issues/8057)\n\\[[parquet](https://redirect.github.com/apache/arrow-rs/labels/parquet)]\n- \\[Variant]: Implement `DataType::Interval` support for\n`cast_to_variant` kernel\n[#&#8203;8056](https://redirect.github.com/apache/arrow-rs/issues/8056)\n\\[[parquet](https://redirect.github.com/apache/arrow-rs/labels/parquet)]\n- \\[Variant]: Implement `DataType::Time32/Time64` support for\n`cast_to_variant` kernel\n[#&#8203;8055](https://redirect.github.com/apache/arrow-rs/issues/8055)\n\\[[parquet](https://redirect.github.com/apache/arrow-rs/labels/parquet)]\n- \\[Variant]: Implement `DataType::Date32 / DataType::Date64` support\nfor `cast_to_variant` kernel\n[#&#8203;8054](https://redirect.github.com/apache/arrow-rs/issues/8054)\n\\[[parquet](https://redirect.github.com/apache/arrow-rs/labels/parquet)]\n- \\[Variant]: Implement `DataType::Null` support for `cast_to_variant`\nkernel\n[#&#8203;8053](https://redirect.github.com/apache/arrow-rs/issues/8053)\n\\[[parquet](https://redirect.github.com/apache/arrow-rs/labels/parquet)]\n- \\[Variant]: Implement `DataType::Boolean` support for\n`cast_to_variant` kernel\n[#&#8203;8052](https://redirect.github.com/apache/arrow-rs/issues/8052)\n\\[[parquet](https://redirect.github.com/apache/arrow-rs/labels/parquet)]\n- \\[Variant]: Implement `DataType::FixedSizeBinary` support for\n`cast_to_variant` kernel\n[#&#8203;8051](https://redirect.github.com/apache/arrow-rs/issues/8051)\n\\[[parquet](https://redirect.github.com/apache/arrow-rs/labels/parquet)]\n- \\[Variant]: Implement `DataType::Binary/LargeBinary/BinaryView`\nsupport for `cast_to_variant` kernel\n[#&#8203;8050](https://redirect.github.com/apache/arrow-rs/issues/8050)\n\\[[parquet](https://redirect.github.com/apache/arrow-rs/labels/parquet)]\n- \\[Variant]: Implement `DataType::Utf8/LargeUtf8/Utf8View` support for\n`cast_to_variant` kernel\n[#&#8203;8049](https://redirect.github.com/apache/arrow-rs/issues/8049)\n\\[[parquet](https://redirect.github.com/apache/arrow-rs/labels/parquet)]\n- \\[Variant] Implement `cast_to_variant` kernel\n[#&#8203;8043](https://redirect.github.com/apache/arrow-rs/issues/8043)\n\\[[parquet](https://redirect.github.com/apache/arrow-rs/labels/parquet)]\n- \\[Variant] Support `variant_get` kernel for shredded variants\n[#&#8203;7941](https://redirect.github.com/apache/arrow-rs/issues/7941)\n\\[[parquet](https://redirect.github.com/apache/arrow-rs/labels/parquet)]\n- Add test for casting `Decimal128` (`i128::MIN` and `i128::MAX`) to\n`f64` with overflow handling\n[#&#8203;7939](https://redirect.github.com/apache/arrow-rs/issues/7939)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n\n**Merged pull requests:**\n\n- \\[Variant] Enhance the variant fuz test to cover time/timestamp/uuid\nprimitive type\n[#&#8203;8200](https://redirect.github.com/apache/arrow-rs/pull/8200)\n([klion26](https://redirect.github.com/klion26))\n- \\[Variant] VariantArrayBuilder tracks only offsets\n[#&#8203;8193](https://redirect.github.com/apache/arrow-rs/pull/8193)\n([scovich](https://redirect.github.com/scovich))\n- \\[Variant] Caller provides ParentState to ValueBuilder methods\n[#&#8203;8189](https://redirect.github.com/apache/arrow-rs/pull/8189)\n([scovich](https://redirect.github.com/scovich))\n- \\[Variant] Rename ValueBuffer as ValueBuilder\n[#&#8203;8187](https://redirect.github.com/apache/arrow-rs/pull/8187)\n([scovich](https://redirect.github.com/scovich))\n- \\[Variant] ParentState handles finish/rollback for builders\n[#&#8203;8185](https://redirect.github.com/apache/arrow-rs/pull/8185)\n([scovich](https://redirect.github.com/scovich))\n- \\[Variant]: Implement `DataType::RunEndEncoded` support for\n`cast_to_variant` kernel\n[#&#8203;8174](https://redirect.github.com/apache/arrow-rs/pull/8174)\n([liamzwbao](https://redirect.github.com/liamzwbao))\n- \\[Variant]: Implement `DataType::Dictionary` support for\n`cast_to_variant` kernel\n[#&#8203;8173](https://redirect.github.com/apache/arrow-rs/pull/8173)\n([liamzwbao](https://redirect.github.com/liamzwbao))\n- Implement `ArrayBuilder` for `UnionBuilder`\n[#&#8203;8169](https://redirect.github.com/apache/arrow-rs/pull/8169)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n([grtlr](https://redirect.github.com/grtlr))\n- \\[Variant] Support `LargeString` and `StringView` in\n`batch_json_string_to_variant`\n[#&#8203;8163](https://redirect.github.com/apache/arrow-rs/pull/8163)\n([liamzwbao](https://redirect.github.com/liamzwbao))\n- \\[Variant] Rename `batch_json_string_to_variant` and\n`batch_variant_to_json_string`\n[#&#8203;8161](https://redirect.github.com/apache/arrow-rs/pull/8161)\n([liamzwbao](https://redirect.github.com/liamzwbao))\n- \\[Variant] Add primitive type timestamp\\_nanos(with\\&without timezone)\nand uuid\n[#&#8203;8149](https://redirect.github.com/apache/arrow-rs/pull/8149)\n([klion26](https://redirect.github.com/klion26))\n- refactor(avro): Use impl Write instead of dyn Write in encoder\n[#&#8203;8148](https://redirect.github.com/apache/arrow-rs/pull/8148)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n([Xuanwo](https://redirect.github.com/Xuanwo))\n- chore: Use tempfile to replace hand-written utils functions\n[#&#8203;8147](https://redirect.github.com/apache/arrow-rs/pull/8147)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n([Xuanwo](https://redirect.github.com/Xuanwo))\n- feat: support push batch direct to completed and add biggest coalesce\nbatch support\n[#&#8203;8146](https://redirect.github.com/apache/arrow-rs/pull/8146)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n([zhuqi-lucas](https://redirect.github.com/zhuqi-lucas))\n- \\[Variant] Add human-readable impl Debug for Variant\n[#&#8203;8140](https://redirect.github.com/apache/arrow-rs/pull/8140)\n([scovich](https://redirect.github.com/scovich))\n- \\[Variant] Fix broken metadata builder rollback\n[#&#8203;8135](https://redirect.github.com/apache/arrow-rs/pull/8135)\n([scovich](https://redirect.github.com/scovich))\n- \\[Variant]: Implement DataType::Interval support for cast\\_to\\_variant\nkernel\n[#&#8203;8125](https://redirect.github.com/apache/arrow-rs/pull/8125)\n([codephage2020](https://redirect.github.com/codephage2020))\n- Add schema resolution and type promotion support to arrow-avro Decoder\n[#&#8203;8124](https://redirect.github.com/apache/arrow-rs/pull/8124)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n([jecsand838](https://redirect.github.com/jecsand838))\n- Add Initial `arrow-avro` writer implementation with basic type support\n[#&#8203;8123](https://redirect.github.com/apache/arrow-rs/pull/8123)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n([jecsand838](https://redirect.github.com/jecsand838))\n- \\[Variant] Add Variant::Time primitive and cast logic\n[#&#8203;8114](https://redirect.github.com/apache/arrow-rs/pull/8114)\n([klion26](https://redirect.github.com/klion26))\n- \\[Variant] Support Timestamp to variant for `cast_to_variant` kernel\n[#&#8203;8113](https://redirect.github.com/apache/arrow-rs/pull/8113)\n([abacef](https://redirect.github.com/abacef))\n- Bump actions/checkout from 4 to 5\n[#&#8203;8110](https://redirect.github.com/apache/arrow-rs/pull/8110)\n([dependabot\\[bot\\]](https://redirect.github.com/apps/dependabot))\n- \\[Varaint]: add `DataType::Null` support to cast\\_to\\_variant\n[#&#8203;8107](https://redirect.github.com/apache/arrow-rs/pull/8107)\n([feniljain](https://redirect.github.com/feniljain))\n- \\[Variant] Adding fixed size byte array to variant and test\n[#&#8203;8106](https://redirect.github.com/apache/arrow-rs/pull/8106)\n([abacef](https://redirect.github.com/abacef))\n- \\[VARIANT] Initial integration tests for variant reads\n[#&#8203;8104](https://redirect.github.com/apache/arrow-rs/pull/8104)\n\\[[parquet](https://redirect.github.com/apache/arrow-rs/labels/parquet)]\n([carpecodeum](https://redirect.github.com/carpecodeum))\n- \\[Variant]: Implement\n`DataType::Decimal32/Decimal64/Decimal128/Decimal256` support for\n`cast_to_variant` kernel\n[#&#8203;8101](https://redirect.github.com/apache/arrow-rs/pull/8101)\n([liamzwbao](https://redirect.github.com/liamzwbao))\n- Refactor arrow-avro `Decoder` to support partial decoding\n[#&#8203;8100](https://redirect.github.com/apache/arrow-rs/pull/8100)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n([jecsand838](https://redirect.github.com/jecsand838))\n- fix: Validate metadata len in IPC reader\n[#&#8203;8097](https://redirect.github.com/apache/arrow-rs/pull/8097)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n([JakeDern](https://redirect.github.com/JakeDern))\n- \\[parquet] further improve logical type compatibility in ArrowWriter\n[#&#8203;8095](https://redirect.github.com/apache/arrow-rs/pull/8095)\n\\[[parquet](https://redirect.github.com/apache/arrow-rs/labels/parquet)]\n([albertlockett](https://redirect.github.com/albertlockett))\n- \\[Varint] Implement ShreddingState::AllNull variant\n[#&#8203;8093](https://redirect.github.com/apache/arrow-rs/pull/8093)\n([codephage2020](https://redirect.github.com/codephage2020))\n- \\[Variant] Minor: Add comments to tickets for follow on items\n[#&#8203;8092](https://redirect.github.com/apache/arrow-rs/pull/8092)\n([alamb](https://redirect.github.com/alamb))\n- \\[VARIANT] Add support for DataType::Struct for cast\\_to\\_variant\n[#&#8203;8090](https://redirect.github.com/apache/arrow-rs/pull/8090)\n([carpecodeum](https://redirect.github.com/carpecodeum))\n- \\[VARIANT] Add support for DataType::Utf8/LargeUtf8/Utf8View for\ncast\\_to\\_variant\n[#&#8203;8089](https://redirect.github.com/apache/arrow-rs/pull/8089)\n([carpecodeum](https://redirect.github.com/carpecodeum))\n- \\[Variant] Implement `DataType::Boolean` support for `cast_to_variant`\nkernel\n[#&#8203;8085](https://redirect.github.com/apache/arrow-rs/pull/8085)\n([sdf-jkl](https://redirect.github.com/sdf-jkl))\n- \\[Variant] Implement `DataType::{Date32,Date64}` => `Variant::Date`\n[#&#8203;8081](https://redirect.github.com/apache/arrow-rs/pull/8081)\n([superserious-dev](https://redirect.github.com/superserious-dev))\n- Fix new clippy lints from Rust 1.89\n[#&#8203;8078](https://redirect.github.com/apache/arrow-rs/pull/8078)\n\\[[parquet](https://redirect.github.com/apache/arrow-rs/labels/parquet)]\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n\\[[arrow-flight](https://redirect.github.com/apache/arrow-rs/labels/arrow-flight)]\n([alamb](https://redirect.github.com/alamb))\n- Implement ArrowSchema to AvroSchema conversion logic in arrow-avro\n[#&#8203;8075](https://redirect.github.com/apache/arrow-rs/pull/8075)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n([jecsand838](https://redirect.github.com/jecsand838))\n- Implement `DataType::{Binary, LargeBinary, BinaryView}` =>\n`Variant::Binary`\n[#&#8203;8074](https://redirect.github.com/apache/arrow-rs/pull/8074)\n([superserious-dev](https://redirect.github.com/superserious-dev))\n- \\[Variant] Implement `DataType::Float16` => `Variant::Float`\n[#&#8203;8073](https://redirect.github.com/apache/arrow-rs/pull/8073)\n([superserious-dev](https://redirect.github.com/superserious-dev))\n- create PageIndexPolicy to allow optional indexes\n[#&#8203;8071](https://redirect.github.com/apache/arrow-rs/pull/8071)\n\\[[parquet](https://redirect.github.com/apache/arrow-rs/labels/parquet)]\n([kczimm](https://redirect.github.com/kczimm))\n- \\[Variant] Minor: use From impl to make conversion infallable\n[#&#8203;8068](https://redirect.github.com/apache/arrow-rs/pull/8068)\n\\[[parquet](https://redirect.github.com/apache/arrow-rs/labels/parquet)]\n([alamb](https://redirect.github.com/alamb))\n- Bump actions/download-artifact from 4 to 5\n[#&#8203;8066](https://redirect.github.com/apache/arrow-rs/pull/8066)\n([dependabot\\[bot\\]](https://redirect.github.com/apps/dependabot))\n- Added arrow-avro schema resolution foundations and type promotion\n[#&#8203;8047](https://redirect.github.com/apache/arrow-rs/pull/8047)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n([jecsand838](https://redirect.github.com/jecsand838))\n- Fix arrow-avro type resolver register bug\n[#&#8203;8046](https://redirect.github.com/apache/arrow-rs/pull/8046)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n([yongkyunlee](https://redirect.github.com/yongkyunlee))\n- implement `cast_to_variant` kernel to cast native types to\n`VariantArray`\n[#&#8203;8044](https://redirect.github.com/apache/arrow-rs/pull/8044)\n\\[[parquet](https://redirect.github.com/apache/arrow-rs/labels/parquet)]\n([alamb](https://redirect.github.com/alamb))\n- Add arrow-avro `SchemaStore` and fingerprinting\n[#&#8203;8039](https://redirect.github.com/apache/arrow-rs/pull/8039)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n([jecsand838](https://redirect.github.com/jecsand838))\n- Add more benchmarks for Parquet thrift decoding\n[#&#8203;8037](https://redirect.github.com/apache/arrow-rs/pull/8037)\n\\[[parquet](https://redirect.github.com/apache/arrow-rs/labels/parquet)]\n([etseidl](https://redirect.github.com/etseidl))\n- Support multi-threaded writing of Parquet files with modular\nencryption\n[#&#8203;8029](https://redirect.github.com/apache/arrow-rs/pull/8029)\n\\[[parquet](https://redirect.github.com/apache/arrow-rs/labels/parquet)]\n([rok](https://redirect.github.com/rok))\n- Add arrow-avro Decoder Benchmarks\n[#&#8203;8025](https://redirect.github.com/apache/arrow-rs/pull/8025)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n([jecsand838](https://redirect.github.com/jecsand838))\n- feat: add method for sync Parquet reader read bloom filter\n[#&#8203;8024](https://redirect.github.com/apache/arrow-rs/pull/8024)\n\\[[parquet](https://redirect.github.com/apache/arrow-rs/labels/parquet)]\n([mapleFU](https://redirect.github.com/mapleFU))\n- \\[Variant] Add `variant_get` and Shredded `VariantArray`\n[#&#8203;8021](https://redirect.github.com/apache/arrow-rs/pull/8021)\n\\[[parquet](https://redirect.github.com/apache/arrow-rs/labels/parquet)]\n([alamb](https://redirect.github.com/alamb))\n- Implement arrow-avro SchemaStore and Fingerprinting To Enable Schema\nResolution\n[#&#8203;8006](https://redirect.github.com/apache/arrow-rs/pull/8006)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n([jecsand838](https://redirect.github.com/jecsand838))\n- \\[Parquet] Add tests for IO/CPU access in parquet reader\n[#&#8203;7971](https://redirect.github.com/apache/arrow-rs/pull/7971)\n\\[[parquet](https://redirect.github.com/apache/arrow-rs/labels/parquet)]\n([alamb](https://redirect.github.com/alamb))\n- Speed up Parquet filter pushdown v4 (Predicate evaluation cache for\nasync\\_reader)\n[#&#8203;7850](https://redirect.github.com/apache/arrow-rs/pull/7850)\n\\[[parquet](https://redirect.github.com/apache/arrow-rs/labels/parquet)]\n([XiangpengHao](https://redirect.github.com/XiangpengHao))\n- Implement cast and other operations on decimal32 and decimal64\n[#&#8203;7815](https://redirect.github.com/apache/arrow-rs/pull/7815)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n([CurtHagenlocher](https://redirect.github.com/CurtHagenlocher))\n\n\\* *This Changelog was automatically generated by\n[github\\_changelog\\_generator](https://redirect.github.com/github-changelog-generator/github-changelog-generator)*\n\n###\n[`v56.0.0`](https://redirect.github.com/apache/arrow-rs/blob/HEAD/CHANGELOG.md#5610-2025-08-21)\n\n[Compare\nSource](https://redirect.github.com/apache/arrow-rs/compare/55.2.0...56.0.0)\n\n[Full\nChangelog](https://redirect.github.com/apache/arrow-rs/compare/56.0.0...56.1.0)\n\n**Implemented enhancements:**\n\n- Implement cast and other operations on decimal32 and decimal64\n[#&#8203;7815](https://redirect.github.com/apache/arrow-rs/issues/7815)\n[#&#8203;8204](https://redirect.github.com/apache/arrow-rs/issues/8204)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n- Speed up Parquet filter pushdown with predicate cache\n[#&#8203;8203](https://redirect.github.com/apache/arrow-rs/issues/8203)\n\\[[parquet](https://redirect.github.com/apache/arrow-rs/labels/parquet)]\n- Optionally read parquet page indexes\n[#&#8203;8070](https://redirect.github.com/apache/arrow-rs/issues/8070)\n\\[[parquet](https://redirect.github.com/apache/arrow-rs/labels/parquet)]\n- Parquet reader: add method for sync reader read bloom filter\n[#&#8203;8023](https://redirect.github.com/apache/arrow-rs/issues/8023)\n\\[[parquet](https://redirect.github.com/apache/arrow-rs/labels/parquet)]\n- \\[parquet] Support writing logically equivalent types to `ArrowWriter`\n[#&#8203;8012](https://redirect.github.com/apache/arrow-rs/issues/8012)\n\\[[parquet](https://redirect.github.com/apache/arrow-rs/labels/parquet)]\n- Improve StringArray(Utf8) sort performance\n[#&#8203;7847](https://redirect.github.com/apache/arrow-rs/issues/7847)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n- feat: arrow-ipc delta dictionary support\n[#&#8203;8001](https://redirect.github.com/apache/arrow-rs/pull/8001)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n([JakeDern](https://redirect.github.com/JakeDern))\n\n**Fixed bugs:**\n\n- The Rustdocs are clean CI job is failing\n[#&#8203;8175](https://redirect.github.com/apache/arrow-rs/issues/8175)\n- \\[avro] Bug in resolving avro schema with named type\n[#&#8203;8045](https://redirect.github.com/apache/arrow-rs/issues/8045)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n- Doc test failure (test arrow-avro/src/lib.rs - reader) when verifying\navro 56.0.0 RC1 release\n[#&#8203;8018](https://redirect.github.com/apache/arrow-rs/issues/8018)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n\n**Documentation updates:**\n\n- arrow-row: Document dictionary handling\n[#&#8203;8168](https://redirect.github.com/apache/arrow-rs/pull/8168)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n([alamb](https://redirect.github.com/alamb))\n- Docs: Clarify that Array::value does not check for nulls\n[#&#8203;8065](https://redirect.github.com/apache/arrow-rs/pull/8065)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n([alamb](https://redirect.github.com/alamb))\n- docs: Fix a typo in README\n[#&#8203;8036](https://redirect.github.com/apache/arrow-rs/pull/8036)\n([EricccTaiwan](https://redirect.github.com/EricccTaiwan))\n- Add more comments to the internal parquet reader\n[#&#8203;7932](https://redirect.github.com/apache/arrow-rs/pull/7932)\n\\[[parquet](https://redirect.github.com/apache/arrow-rs/labels/parquet)]\n([alamb](https://redirect.github.com/alamb))\n\n**Performance improvements:**\n\n- perf(arrow-ipc): avoid counting nulls in `RecordBatchDecoder`\n[#&#8203;8127](https://redirect.github.com/apache/arrow-rs/pull/8127)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n([rluvaton](https://redirect.github.com/rluvaton))\n- Use `Vec` directly in builders\n[#&#8203;7984](https://redirect.github.com/apache/arrow-rs/pull/7984)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n([liamzwbao](https://redirect.github.com/liamzwbao))\n- Improve StringArray(Utf8) sort performance (\\~2-4x faster)\n[#&#8203;7860](https://redirect.github.com/apache/arrow-rs/pull/7860)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n([zhuqi-lucas](https://redirect.github.com/zhuqi-lucas))\n\n**Closed issues:**\n\n- \\[Variant] Improve fuzz test for Variant\n[#&#8203;8199](https://redirect.github.com/apache/arrow-rs/issues/8199)\n- \\[Variant] Improve fuzz test for Variant\n[#&#8203;8198](https://redirect.github.com/apache/arrow-rs/issues/8198)\n- `VariantArrayBuilder` tracks starting offsets instead of (offset, len)\npairs\n[#&#8203;8192](https://redirect.github.com/apache/arrow-rs/issues/8192)\n- Rework `ValueBuilder` API to work with `ParentState` for reliable\nnested rollbacks\n[#&#8203;8188](https://redirect.github.com/apache/arrow-rs/issues/8188)\n- \\[Variant] Rename `ValueBuffer` as `ValueBuilder`\n[#&#8203;8186](https://redirect.github.com/apache/arrow-rs/issues/8186)\n- \\[Variant] Refactor `ParentState` to track and rollback state on\nbehalf of its owning builder\n[#&#8203;8182](https://redirect.github.com/apache/arrow-rs/issues/8182)\n- \\[Variant] `ObjectBuilder` should detect duplicates at insertion time,\nnot at finish\n[#&#8203;8180](https://redirect.github.com/apache/arrow-rs/issues/8180)\n- \\[Variant] ObjectBuilder does not reliably check for duplicates\n[#&#8203;8170](https://redirect.github.com/apache/arrow-rs/issues/8170)\n- \\[Variant] Support `StringView` and `LargeString` in\n´batch\\_json\\_string\\_to\\_variant\\`\n[#&#8203;8145](https://redirect.github.com/apache/arrow-rs/issues/8145)\n\\[[parquet](https://redirect.github.com/apache/arrow-rs/labels/parquet)]\n- \\[Variant] Rename `batch_json_string_to_variant` and\n`batch_variant_to_json_string` json\\_to\\_variant\n[#&#8203;8144](https://redirect.github.com/apache/arrow-rs/issues/8144)\n\\[[parquet](https://redirect.github.com/apache/arrow-rs/labels/parquet)]\n- \\[avro] Use `tempfile` crate rather than custom temporary file\ngenerator in tests\n[#&#8203;8143](https://redirect.github.com/apache/arrow-rs/issues/8143)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n- \\[Avro] Use `Write` rather `dyn Write` in Decoder\n[#&#8203;8142](https://redirect.github.com/apache/arrow-rs/issues/8142)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n- \\[Variant] Nested builder rollback is broken\n[#&#8203;8136](https://redirect.github.com/apache/arrow-rs/issues/8136)\n- \\[Variant] Add support the remaing primitive\ntype(timestamp\\_nanos/timestampntz\\_nanos/uuid) for parquet variant\n[#&#8203;8126](https://redirect.github.com/apache/arrow-rs/issues/8126)\n- Meta: Implement missing Arrow 56.0 lint rules - Sequential workflow\n[#&#8203;8121](https://redirect.github.com/apache/arrow-rs/issues/8121)\n- ARROW-012-015: Add linter rules for remaining Arrow 56.0 breaking\nchanges\n[#&#8203;8120](https://redirect.github.com/apache/arrow-rs/issues/8120)\n- ARROW-010 & ARROW-011: Add linter rules for Parquet Statistics and\nMetadata API removals\n[#&#8203;8119](https://redirect.github.com/apache/arrow-rs/issues/8119)\n- ARROW-009: Add linter rules for IPC Dictionary API removals in Arrow\n56.0\n[#&#8203;8118](https://redirect.github.com/apache/arrow-rs/issues/8118)\n- ARROW-008: Add linter rule for SerializedPageReaderState usize→u64\nbreaking change\n[#&#8203;8117](https://redirect.github.com/apache/arrow-rs/issues/8117)\n- ARROW-007: Add linter rule for Schema.all\\_fields() removal in Arrow\n56.0\n[#&#8203;8116](https://redirect.github.com/apache/arrow-rs/issues/8116)\n- \\[Variant] Implement `ShreddingState::AllNull` variant\n[#&#8203;8088](https://redirect.github.com/apache/arrow-rs/issues/8088)\n\\[[parquet](https://redirect.github.com/apache/arrow-rs/labels/parquet)]\n- \\[Variant] Support Shredded Objects in `variant_get`\n[#&#8203;8083](https://redirect.github.com/apache/arrow-rs/issues/8083)\n\\[[parquet](https://redirect.github.com/apache/arrow-rs/labels/parquet)]\n- \\[Variant]: Implement `DataType::RunEndEncoded` support for\n`cast_to_variant` kernel\n[#&#8203;8064](https://redirect.github.com/apache/arrow-rs/issues/8064)\n\\[[parquet](https://redirect.github.com/apache/arrow-rs/labels/parquet)]\n- \\[Variant]: Implement `DataType::Dictionary` support for\n`cast_to_variant` kernel\n[#&#8203;8062](https://redirect.github.com/apache/arrow-rs/issues/8062)\n\\[[parquet](https://redirect.github.com/apache/arrow-rs/labels/parquet)]\n- \\[Variant]: Implement `DataType::Struct` support for `cast_to_variant`\nkernel\n[#&#8203;8061](https://redirect.github.com/apache/arrow-rs/issues/8061)\n\\[[parquet](https://redirect.github.com/apache/arrow-rs/labels/parquet)]\n- \\[Variant]: Implement\n`DataType::Decimal32/Decimal64/Decimal128/Decimal256` support for\n`cast_to_variant` kernel\n[#&#8203;8059](https://redirect.github.com/apache/arrow-rs/issues/8059)\n\\[[parquet](https://redirect.github.com/apache/arrow-rs/labels/parquet)]\n- \\[Variant]: Implement `DataType::Timestamp(..)` support for\n`cast_to_variant` kernel\n[#&#8203;8058](https://redirect.github.com/apache/arrow-rs/issues/8058)\n\\[[parquet](https://redirect.github.com/apache/arrow-rs/labels/parquet)]\n- \\[Variant]: Implement `DataType::Float16` support for\n`cast_to_variant` kernel\n[#&#8203;8057](https://redirect.github.com/apache/arrow-rs/issues/8057)\n\\[[parquet](https://redirect.github.com/apache/arrow-rs/labels/parquet)]\n- \\[Variant]: Implement `DataType::Interval` support for\n`cast_to_variant` kernel\n[#&#8203;8056](https://redirect.github.com/apache/arrow-rs/issues/8056)\n\\[[parquet](https://redirect.github.com/apache/arrow-rs/labels/parquet)]\n- \\[Variant]: Implement `DataType::Time32/Time64` support for\n`cast_to_variant` kernel\n[#&#8203;8055](https://redirect.github.com/apache/arrow-rs/issues/8055)\n\\[[parquet](https://redirect.github.com/apache/arrow-rs/labels/parquet)]\n- \\[Variant]: Implement `DataType::Date32 / DataType::Date64` support\nfor `cast_to_variant` kernel\n[#&#8203;8054](https://redirect.github.com/apache/arrow-rs/issues/8054)\n\\[[parquet](https://redirect.github.com/apache/arrow-rs/labels/parquet)]\n- \\[Variant]: Implement `DataType::Null` support for `cast_to_variant`\nkernel\n[#&#8203;8053](https://redirect.github.com/apache/arrow-rs/issues/8053)\n\\[[parquet](https://redirect.github.com/apache/arrow-rs/labels/parquet)]\n- \\[Variant]: Implement `DataType::Boolean` support for\n`cast_to_variant` kernel\n[#&#8203;8052](https://redirect.github.com/apache/arrow-rs/issues/8052)\n\\[[parquet](https://redirect.github.com/apache/arrow-rs/labels/parquet)]\n- \\[Variant]: Implement `DataType::FixedSizeBinary` support for\n`cast_to_variant` kernel\n[#&#8203;8051](https://redirect.github.com/apache/arrow-rs/issues/8051)\n\\[[parquet](https://redirect.github.com/apache/arrow-rs/labels/parquet)]\n- \\[Variant]: Implement `DataType::Binary/LargeBinary/BinaryView`\nsupport for `cast_to_variant` kernel\n[#&#8203;8050](https://redirect.github.com/apache/arrow-rs/issues/8050)\n\\[[parquet](https://redirect.github.com/apache/arrow-rs/labels/parquet)]\n- \\[Variant]: Implement `DataType::Utf8/LargeUtf8/Utf8View` support for\n`cast_to_variant` kernel\n[#&#8203;8049](https://redirect.github.com/apache/arrow-rs/issues/8049)\n\\[[parquet](https://redirect.github.com/apache/arrow-rs/labels/parquet)]\n- \\[Variant] Implement `cast_to_variant` kernel\n[#&#8203;8043](https://redirect.github.com/apache/arrow-rs/issues/8043)\n\\[[parquet](https://redirect.github.com/apache/arrow-rs/labels/parquet)]\n- \\[Variant] Support `variant_get` kernel for shredded variants\n[#&#8203;7941](https://redirect.github.com/apache/arrow-rs/issues/7941)\n\\[[parquet](https://redirect.github.com/apache/arrow-rs/labels/parquet)]\n- Add test for casting `Decimal128` (`i128::MIN` and `i128::MAX`) to\n`f64` with overflow handling\n[#&#8203;7939](https://redirect.github.com/apache/arrow-rs/issues/7939)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n\n**Merged pull requests:**\n\n- \\[Variant] Enhance the variant fuz test to cover time/timestamp/uuid\nprimitive type\n[#&#8203;8200](https://redirect.github.com/apache/arrow-rs/pull/8200)\n([klion26](https://redirect.github.com/klion26))\n- \\[Variant] VariantArrayBuilder tracks only offsets\n[#&#8203;8193](https://redirect.github.com/apache/arrow-rs/pull/8193)\n([scovich](https://redirect.github.com/scovich))\n- \\[Variant] Caller provides ParentState to ValueBuilder methods\n[#&#8203;8189](https://redirect.github.com/apache/arrow-rs/pull/8189)\n([scovich](https://redirect.github.com/scovich))\n- \\[Variant] Rename ValueBuffer as ValueBuilder\n[#&#8203;8187](https://redirect.github.com/apache/arrow-rs/pull/8187)\n([scovich](https://redirect.github.com/scovich))\n- \\[Variant] ParentState handles finish/rollback for builders\n[#&#8203;8185](https://redirect.github.com/apache/arrow-rs/pull/8185)\n([scovich](https://redirect.github.com/scovich))\n- \\[Variant]: Implement `DataType::RunEndEncoded` support for\n`cast_to_variant` kernel\n[#&#8203;8174](https://redirect.github.com/apache/arrow-rs/pull/8174)\n([liamzwbao](https://redirect.github.com/liamzwbao))\n- \\[Variant]: Implement `DataType::Dictionary` support for\n`cast_to_variant` kernel\n[#&#8203;8173](https://redirect.github.com/apache/arrow-rs/pull/8173)\n([liamzwbao](https://redirect.github.com/liamzwbao))\n- Implement `ArrayBuilder` for `UnionBuilder`\n[#&#8203;8169](https://redirect.github.com/apache/arrow-rs/pull/8169)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n([grtlr](https://redirect.github.com/grtlr))\n- \\[Variant] Support `LargeString` and `StringView` in\n`batch_json_string_to_variant`\n[#&#8203;8163](https://redirect.github.com/apache/arrow-rs/pull/8163)\n([liamzwbao](https://redirect.github.com/liamzwbao))\n- \\[Variant] Rename `batch_json_string_to_variant` and\n`batch_variant_to_json_string`\n[#&#8203;8161](https://redirect.github.com/apache/arrow-rs/pull/8161)\n([liamzwbao](https://redirect.github.com/liamzwbao))\n- \\[Variant] Add primitive type timestamp\\_nanos(with\\&without timezone)\nand uuid\n[#&#8203;8149](https://redirect.github.com/apache/arrow-rs/pull/8149)\n([klion26](https://redirect.github.com/klion26))\n- refactor(avro): Use impl Write instead of dyn Write in encoder\n[#&#8203;8148](https://redirect.github.com/apache/arrow-rs/pull/8148)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n([Xuanwo](https://redirect.github.com/Xuanwo))\n- chore: Use tempfile to replace hand-written utils functions\n[#&#8203;8147](https://redirect.github.com/apache/arrow-rs/pull/8147)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n([Xuanwo](https://redirect.github.com/Xuanwo))\n- feat: support push batch direct to completed and add biggest coalesce\nbatch support\n[#&#8203;8146](https://redirect.github.com/apache/arrow-rs/pull/8146)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n([zhuqi-lucas](https://redirect.github.com/zhuqi-lucas))\n- \\[Variant] Add human-readable impl Debug for Variant\n[#&#8203;8140](https://redirect.github.com/apache/arrow-rs/pull/8140)\n([scovich](https://redirect.github.com/scovich))\n- \\[Variant] Fix broken metadata builder rollback\n[#&#8203;8135](https://redirect.github.com/apache/arrow-rs/pull/8135)\n([scovich](https://redirect.github.com/scovich))\n- \\[Variant]: Implement DataType::Interval support for cast\\_to\\_variant\nkernel\n[#&#8203;8125](https://redirect.github.com/apache/arrow-rs/pull/8125)\n([codephage2020](https://redirect.github.com/codephage2020))\n- Add schema resolution and type promotion support to arrow-avro Decoder\n[#&#8203;8124](https://redirect.github.com/apache/arrow-rs/pull/8124)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n([jecsand838](https://redirect.github.com/jecsand838))\n- Add Initial `arrow-avro` writer implementation with basic type support\n[#&#8203;8123](https://redirect.github.com/apache/arrow-rs/pull/8123)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n([jecsand838](https://redirect.github.com/jecsand838))\n- \\[Variant] Add Variant::Time primitive and cast logic\n[#&#8203;8114](https://redirect.github.com/apache/arrow-rs/pull/8114)\n([klion26](https://redirect.github.com/klion26))\n- \\[Variant] Support Timestamp to variant for `cast_to_variant` kernel\n[#&#8203;8113](https://redirect.github.com/apache/arrow-rs/pull/8113)\n([abacef](https://redirect.github.com/abacef))\n- Bump actions/checkout from 4 to 5\n[#&#8203;8110](https://redirect.github.com/apache/arrow-rs/pull/8110)\n([dependabot\\[bot\\]](https://redirect.github.com/apps/dependabot))\n- \\[Varaint]: add `DataType::Null` support to cast\\_to\\_variant\n[#&#8203;8107](https://redirect.github.com/apache/arrow-rs/pull/8107)\n([feniljain](https://redirect.github.com/feniljain))\n- \\[Variant] Adding fixed size byte array to variant and test\n[#&#8203;8106](https://redirect.github.com/apache/arrow-rs/pull/8106)\n([abacef](https://redirect.github.com/abacef))\n- \\[VARIANT] Initial integration tests for variant reads\n[#&#8203;8104](https://redirect.github.com/apache/arrow-rs/pull/8104)\n\\[[parquet](https://redirect.github.com/apache/arrow-rs/labels/parquet)]\n([carpecodeum](https://redirect.github.com/carpecodeum))\n- \\[Variant]: Implement\n`DataType::Decimal32/Decimal64/Decimal128/Decimal256` support for\n`cast_to_variant` kernel\n[#&#8203;8101](https://redirect.github.com/apache/arrow-rs/pull/8101)\n([liamzwbao](https://redirect.github.com/liamzwbao))\n- Refactor arrow-avro `Decoder` to support partial decoding\n[#&#8203;8100](https://redirect.github.com/apache/arrow-rs/pull/8100)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n([jecsand838](https://redirect.github.com/jecsand838))\n- fix: Validate metadata len in IPC reader\n[#&#8203;8097](https://redirect.github.com/apache/arrow-rs/pull/8097)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n([JakeDern](https://redirect.github.com/JakeDern))\n- \\[parquet] further improve logical type compatibility in ArrowWriter\n[#&#8203;8095](https://redirect.github.com/apache/arrow-rs/pull/8095)\n\\[[parquet](https://redirect.github.com/apache/arrow-rs/labels/parquet)]\n([albertlockett](https://redirect.github.com/albertlockett))\n- \\[Varint] Implement ShreddingState::AllNull variant\n[#&#8203;8093](https://redirect.github.com/apache/arrow-rs/pull/8093)\n([codephage2020](https://redirect.github.com/codephage2020))\n- \\[Variant] Minor: Add comments to tickets for follow on items\n[#&#8203;8092](https://redirect.github.com/apache/arrow-rs/pull/8092)\n([alamb](https://redirect.github.com/alamb))\n- \\[VARIANT] Add support for DataType::Struct for cast\\_to\\_variant\n[#&#8203;8090](https://redirect.github.com/apache/arrow-rs/pull/8090)\n([carpecodeum](https://redirect.github.com/carpecodeum))\n- \\[VARIANT] Add support for DataType::Utf8/LargeUtf8/Utf8View for\ncast\\_to\\_variant\n[#&#8203;8089](https://redirect.github.com/apache/arrow-rs/pull/8089)\n([carpecodeum](https://redirect.github.com/carpecodeum))\n- \\[Variant] Implement `DataType::Boolean` support for `cast_to_variant`\nkernel\n[#&#8203;8085](https://redirect.github.com/apache/arrow-rs/pull/8085)\n([sdf-jkl](https://redirect.github.com/sdf-jkl))\n- \\[Variant] Implement `DataType::{Date32,Date64}` => `Variant::Date`\n[#&#8203;8081](https://redirect.github.com/apache/arrow-rs/pull/8081)\n([superserious-dev](https://redirect.github.com/superserious-dev))\n- Fix new clippy lints from Rust 1.89\n[#&#8203;8078](https://redirect.github.com/apache/arrow-rs/pull/8078)\n\\[[parquet](https://redirect.github.com/apache/arrow-rs/labels/parquet)]\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n\\[[arrow-flight](https://redirect.github.com/apache/arrow-rs/labels/arrow-flight)]\n([alamb](https://redirect.github.com/alamb))\n- Implement ArrowSchema to AvroSchema conversion logic in arrow-avro\n[#&#8203;8075](https://redirect.github.com/apache/arrow-rs/pull/8075)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n([jecsand838](https://redirect.github.com/jecsand838))\n- Implement `DataType::{Binary, LargeBinary, BinaryView}` =>\n`Variant::Binary`\n[#&#8203;8074](https://redirect.github.com/apache/arrow-rs/pull/8074)\n([superserious-dev](https://redirect.github.com/superserious-dev))\n- \\[Variant] Implement `DataType::Float16` => `Variant::Float`\n[#&#8203;8073](https://redirect.github.com/apache/arrow-rs/pull/8073)\n([superserious-dev](https://redirect.github.com/superserious-dev))\n- create PageIndexPolicy to allow optional indexes\n[#&#8203;8071](https://redirect.github.com/apache/arrow-rs/pull/8071)\n\\[[parquet](https://redirect.github.com/apache/arrow-rs/labels/parquet)]\n([kczimm](https://redirect.github.com/kczimm))\n- \\[Variant] Minor: use From impl to make conversion infallable\n[#&#8203;8068](https://redirect.github.com/apache/arrow-rs/pull/8068)\n\\[[parquet](https://redirect.github.com/apache/arrow-rs/labels/parquet)]\n([alamb](https://redirect.github.com/alamb))\n- Bump actions/download-artifact from 4 to 5\n[#&#8203;8066](https://redirect.github.com/apache/arrow-rs/pull/8066)\n([dependabot\\[bot\\]](https://redirect.github.com/apps/dependabot))\n- Added arrow-avro schema resolution foundations and type promotion\n[#&#8203;8047](https://redirect.github.com/apache/arrow-rs/pull/8047)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n([jecsand838](https://redirect.github.com/jecsand838))\n- Fix arrow-avro type resolver register bug\n[#&#8203;8046](https://redirect.github.com/apache/arrow-rs/pull/8046)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n([yongkyunlee](https://redirect.github.com/yongkyunlee))\n- implement `cast_to_variant` kernel to cast native types to\n`VariantArray`\n[#&#8203;8044](https://redirect.github.com/apache/arrow-rs/pull/8044)\n\\[[parquet](https://redirect.github.com/apache/arrow-rs/labels/parquet)]\n([alamb](https://redirect.github.com/alamb))\n- Add arrow-avro `SchemaStore` and fingerprinting\n[#&#8203;8039](https://redirect.github.com/apache/arrow-rs/pull/8039)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n([jecsand838](https://redirect.github.com/jecsand838))\n- Add more benchmarks for Parquet thrift decoding\n[#&#8203;8037](https://redirect.github.com/apache/arrow-rs/pull/8037)\n\\[[parquet](https://redirect.github.com/apache/arrow-rs/labels/parquet)]\n([etseidl](https://redirect.github.com/etseidl))\n- Support multi-threaded writing of Parquet files with modular\nencryption\n[#&#8203;8029](https://redirect.github.com/apache/arrow-rs/pull/8029)\n\\[[parquet](https://redirect.github.com/apache/arrow-rs/labels/parquet)]\n([rok](https://redirect.github.com/rok))\n- Add arrow-avro Decoder Benchmarks\n[#&#8203;8025](https://redirect.github.com/apache/arrow-rs/pull/8025)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n([jecsand838](https://redirect.github.com/jecsand838))\n- feat: add method for sync Parquet reader read bloom filter\n[#&#8203;8024](https://redirect.github.com/apache/arrow-rs/pull/8024)\n\\[[parquet](https://redirect.github.com/apache/arrow-rs/labels/parquet)]\n([mapleFU](https://redirect.github.com/mapleFU))\n- \\[Variant] Add `variant_get` and Shredded `VariantArray`\n[#&#8203;8021](https://redirect.github.com/apache/arrow-rs/pull/8021)\n\\[[parquet](https://redirect.github.com/apache/arrow-rs/labels/parquet)]\n([alamb](https://redirect.github.com/alamb))\n- Implement arrow-avro SchemaStore and Fingerprinting To Enable Schema\nResolution\n[#&#8203;8006](https://redirect.github.com/apache/arrow-rs/pull/8006)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n([jecsand838](https://redirect.github.com/jecsand838))\n- \\[Parquet] Add tests for IO/CPU access in parquet reader\n[#&#8203;7971](https://redirect.github.com/apache/arrow-rs/pull/7971)\n\\[[parquet](https://redirect.github.com/apache/arrow-rs/labels/parquet)]\n([alamb](https://redirect.github.com/alamb))\n- Speed up Parquet filter pushdown v4 (Predicate evaluation cache for\nasync\\_reader)\n[#&#8203;7850](https://redirect.github.com/apache/arrow-rs/pull/7850)\n\\[[parquet](https://redirect.github.com/apache/arrow-rs/labels/parquet)]\n([XiangpengHao](https://redirect.github.com/XiangpengHao))\n- Implement cast and other operations on decimal32 and decimal64\n[#&#8203;7815](https://redirect.github.com/apache/arrow-rs/pull/7815)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n([CurtHagenlocher](https://redirect.github.com/CurtHagenlocher))\n\n\\* *This Changelog was automatically generated by\n[github\\_changelog\\_generator](https://redirect.github.com/github-changelog-generator/github-changelog-generator)*\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - \"before 8am on Monday\" (UTC),\nAutomerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0MS44Mi43IiwidXBkYXRlZEluVmVyIjoiNDEuODIuNyIsInRhcmdldEJyYW5jaCI6Im1haW4iLCJsYWJlbHMiOlsiZGVwZW5kZW5jaWVzIl19-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2025-08-25T15:16:14Z",
          "tree_id": "0e2c4777d943c42a74fc55547cce0f63274ee2ca",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/b0f3afb669bb7c2e877953ac984ae4d1817d340b"
        },
        "date": 1756140436654,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 735000,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 22050000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 22050000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-avg",
            "value": 5.61,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 6.69,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 160.53,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 189.04,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 738166.6666666666,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 22145000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 22145000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 5.61,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 6.71,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 128.32,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 147.1,
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
          "id": "a053209dcd58d590c1cfaee02fbd76d1f010c99b",
          "message": "Move retry_processor from engine to otap-df-otap to use in OtapPdata context (#977)\n\nPart of #783 \nPart of #888 \n\nMoves the retry_processor from `engine` to `otap`. Adds a factory and a\nfew test helpers.\nReviewer: is there an existing location for such helpers, or similar\nhelpers to my new `num_rows()` function? How should we test OtapPdata\nequivalence?",
          "timestamp": "2025-08-25T17:01:20Z",
          "tree_id": "12cd56d1458e463a59b0afb6652f4c07ec418e98",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/a053209dcd58d590c1cfaee02fbd76d1f010c99b"
        },
        "date": 1756142468032,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 739166.6666666666,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 22175000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 22175000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-avg",
            "value": 5.71,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 6.68,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 159.65,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 189.71,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 741333.3333333334,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 22240000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 22240000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 5.7,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 7.08,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 135.33,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 192.84,
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
          "id": "e83a326c8dce36f2eab7cbb3d87eff7e6d53b429",
          "message": "[query-engine] Fix bugs when performing assignment using defined names (#998)\n\nRelates to #995\n\n## Changes\n\n* Fixes bugs when performing assignment in `project` & `extend`\nstatements when the target name matches a variable or attached data",
          "timestamp": "2025-08-25T17:16:08Z",
          "tree_id": "13d397dd185f2e29043fa85d9fd6c0c45a7c27d2",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/e83a326c8dce36f2eab7cbb3d87eff7e6d53b429"
        },
        "date": 1756146325913,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 743166.6666666666,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 22295000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 22295000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-avg",
            "value": 5.89,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 6.86,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 167.16,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 204.49,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 739000,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 22170000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 22170000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 5.66,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 6.59,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 124.48,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 143.86,
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
          "distinct": true,
          "id": "f54f0ff42afaa4fd58a3c9e6650bdea42ae5a6fb",
          "message": "[otap-dataflow] [otlp]: validate batch_processor config and register processor factory (#992)\n\nFixes  #965",
          "timestamp": "2025-08-25T19:12:11Z",
          "tree_id": "f8d221a8353946115aae911dc2e0495b217cf9e1",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/f54f0ff42afaa4fd58a3c9e6650bdea42ae5a6fb"
        },
        "date": 1756149636130,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 740500,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 22215000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 22215000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-avg",
            "value": 5.72,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 6.63,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 159.38,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 193.79,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 748500,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 22455000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 22455000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 5.73,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 6.79,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 131.19,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 156.43,
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
          "id": "52cdec048587047bc85c4b7940e8654d2fb5834f",
          "message": "[query-engine] Allow accessor expressions in KQL group by expressions (#1009)\n\n## Changes\n\n* Improve the user experience when using KQL `summarize` by allowing\naccessor expressions in group-by clauses with auto resolution of names",
          "timestamp": "2025-08-25T23:27:27Z",
          "tree_id": "7f0188beb068ca7afa68e9ad802e4c367d88474b",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/52cdec048587047bc85c4b7940e8654d2fb5834f"
        },
        "date": 1756165040009,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 743166.6666666666,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 22295000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 22295000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-avg",
            "value": 5.82,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 6.83,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 164.85,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 190.29,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 738333.3333333334,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 22150000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 22150000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 5.53,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 6.56,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 128.12,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 154.47,
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
          "id": "110b3e365c85ae517ab944c9d93de5d9716a4d3e",
          "message": "Update github workflow dependencies (#999)\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n|---|---|---|---|\n|\n[codecov/codecov-action](https://redirect.github.com/codecov/codecov-action)\n| action | minor | `v5.4.3` -> `v5.5.0` |\n|\n[github/codeql-action](https://redirect.github.com/github/codeql-action)\n| action | patch | `v3.29.10` -> `v3.29.11` |\n| [korandoru/hawkeye](https://redirect.github.com/korandoru/hawkeye) |\naction | minor | `v6.1.1` -> `v6.2.0` |\n|\n[taiki-e/install-action](https://redirect.github.com/taiki-e/install-action)\n| action | patch | `v2.58.17` -> `v2.58.21` |\n\n---\n\n### Release Notes\n\n<details>\n<summary>codecov/codecov-action (codecov/codecov-action)</summary>\n\n###\n[`v5.5.0`](https://redirect.github.com/codecov/codecov-action/blob/HEAD/CHANGELOG.md#v550)\n\n[Compare\nSource](https://redirect.github.com/codecov/codecov-action/compare/v5.4.3...v5.5.0)\n\n##### What's Changed\n\n- feat: upgrade wrapper to 0.2.4 by\n[@&#8203;jviall](https://redirect.github.com/jviall) in\n[#&#8203;1864](https://redirect.github.com/codecov/codecov-action/pull/1864)\n- Pin actions/github-script by Git SHA by\n[@&#8203;martincostello](https://redirect.github.com/martincostello) in\n[#&#8203;1859](https://redirect.github.com/codecov/codecov-action/pull/1859)\n- fix: check reqs exist by\n[@&#8203;joseph-sentry](https://redirect.github.com/joseph-sentry) in\n[#&#8203;1835](https://redirect.github.com/codecov/codecov-action/pull/1835)\n- fix: Typo in README by\n[@&#8203;spalmurray](https://redirect.github.com/spalmurray) in\n[#&#8203;1838](https://redirect.github.com/codecov/codecov-action/pull/1838)\n- docs: Refine OIDC docs by\n[@&#8203;spalmurray](https://redirect.github.com/spalmurray) in\n[#&#8203;1837](https://redirect.github.com/codecov/codecov-action/pull/1837)\n- build(deps): bump github/codeql-action from 3.28.17 to 3.28.18 by\n[@&#8203;app/dependabot](https://redirect.github.com/app/dependabot) in\n[#&#8203;1829](https://redirect.github.com/codecov/codecov-action/pull/1829)\n\n**Full Changelog**:\n<https://github.com/codecov/codecov-action/compare/v5.4.3..v5.5.0>\n\n</details>\n\n<details>\n<summary>github/codeql-action (github/codeql-action)</summary>\n\n###\n[`v3.29.11`](https://redirect.github.com/github/codeql-action/compare/v3.29.10...v3.29.11)\n\n[Compare\nSource](https://redirect.github.com/github/codeql-action/compare/v3.29.10...v3.29.11)\n\n</details>\n\n<details>\n<summary>korandoru/hawkeye (korandoru/hawkeye)</summary>\n\n###\n[`v6.2.0`](https://redirect.github.com/korandoru/hawkeye/releases/tag/v6.2.0)\n\n[Compare\nSource](https://redirect.github.com/korandoru/hawkeye/compare/v6.1.1...v6.2.0)\n\n#### Install hawkeye 6.2.0\n\n##### Install prebuilt binaries via shell script\n\n```sh\ncurl --proto '=https' --tlsv1.2 -LsSf https://github.com/korandoru/hawkeye/releases/download/v6.2.0/hawkeye-installer.sh | sh\n```\n\n#### Download hawkeye 6.2.0\n\n| File | Platform | Checksum |\n|\n--------------------------------------------------------------------------------------------------------------------------------------------------\n| ------------------- |\n-------------------------------------------------------------------------------------------------------------------------\n|\n|\n[hawkeye-aarch64-apple-darwin.tar.xz](https://redirect.github.com/korandoru/hawkeye/releases/download/v6.2.0/hawkeye-aarch64-apple-darwin.tar.xz)\n| Apple Silicon macOS |\n[checksum](https://redirect.github.com/korandoru/hawkeye/releases/download/v6.2.0/hawkeye-aarch64-apple-darwin.tar.xz.sha256)\n|\n|\n[hawkeye-x86\\_64-apple-darwin.tar.xz](https://redirect.github.com/korandoru/hawkeye/releases/download/v6.2.0/hawkeye-x86_64-apple-darwin.tar.xz)\n| Intel macOS |\n[checksum](https://redirect.github.com/korandoru/hawkeye/releases/download/v6.2.0/hawkeye-x86_64-apple-darwin.tar.xz.sha256)\n|\n|\n[hawkeye-x86\\_64-pc-windows-msvc.zip](https://redirect.github.com/korandoru/hawkeye/releases/download/v6.2.0/hawkeye-x86_64-pc-windows-msvc.zip)\n| x64 Windows |\n[checksum](https://redirect.github.com/korandoru/hawkeye/releases/download/v6.2.0/hawkeye-x86_64-pc-windows-msvc.zip.sha256)\n|\n|\n[hawkeye-aarch64-unknown-linux-gnu.tar.xz](https://redirect.github.com/korandoru/hawkeye/releases/download/v6.2.0/hawkeye-aarch64-unknown-linux-gnu.tar.xz)\n| ARM64 Linux |\n[checksum](https://redirect.github.com/korandoru/hawkeye/releases/download/v6.2.0/hawkeye-aarch64-unknown-linux-gnu.tar.xz.sha256)\n|\n|\n[hawkeye-x86\\_64-unknown-linux-gnu.tar.xz](https://redirect.github.com/korandoru/hawkeye/releases/download/v6.2.0/hawkeye-x86_64-unknown-linux-gnu.tar.xz)\n| x64 Linux |\n[checksum](https://redirect.github.com/korandoru/hawkeye/releases/download/v6.2.0/hawkeye-x86_64-unknown-linux-gnu.tar.xz.sha256)\n|\n\n</details>\n\n<details>\n<summary>taiki-e/install-action (taiki-e/install-action)</summary>\n\n###\n[`v2.58.21`](https://redirect.github.com/taiki-e/install-action/blob/HEAD/CHANGELOG.md#100---2021-12-30)\n\n[Compare\nSource](https://redirect.github.com/taiki-e/install-action/compare/v2.58.20...v2.58.21)\n\nInitial release\n\n[Unreleased]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.21...HEAD\n\n[2.58.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.20...v2.58.21\n\n[2.58.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.19...v2.58.20\n\n[2.58.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.18...v2.58.19\n\n[2.58.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.17...v2.58.18\n\n[2.58.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.16...v2.58.17\n\n[2.58.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.15...v2.58.16\n\n[2.58.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.14...v2.58.15\n\n[2.58.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.13...v2.58.14\n\n[2.58.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.12...v2.58.13\n\n[2.58.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.11...v2.58.12\n\n[2.58.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.10...v2.58.11\n\n[2.58.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.9...v2.58.10\n\n[2.58.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.8...v2.58.9\n\n[2.58.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.7...v2.58.8\n\n[2.58.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.6...v2.58.7\n\n[2.58.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.5...v2.58.6\n\n[2.58.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.4...v2.58.5\n\n[2.58.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.3...v2.58.4\n\n[2.58.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.2...v2.58.3\n\n[2.58.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.1...v2.58.2\n\n[2.58.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.0...v2.58.1\n\n[2.58.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.8...v2.58.0\n\n[2.57.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.7...v2.57.8\n\n[2.57.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.6...v2.57.7\n\n[2.57.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.5...v2.57.6\n\n[2.57.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.4...v2.57.5\n\n[2.57.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.3...v2.57.4\n\n[2.57.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.2...v2.57.3\n\n[2.57.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.1...v2.57.2\n\n[2.57.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.0...v2.57.1\n\n[2.57.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.24...v2.57.0\n\n[2.56.24]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.23...v2.56.24\n\n[2.56.23]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.22...v2.56.23\n\n[2.56.22]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.21...v2.56.22\n\n[2.56.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.20...v2.56.21\n\n[2.56.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.19...v2.56.20\n\n[2.56.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.18...v2.56.19\n\n[2.56.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.17...v2.56.18\n\n[2.56.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.16...v2.56.17\n\n[2.56.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.15...v2.56.16\n\n[2.56.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.14...v2.56.15\n\n[2.56.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.13...v2.56.14\n\n[2.56.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.12...v2.56.13\n\n[2.56.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.11...v2.56.12\n\n[2.56.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.10...v2.56.11\n\n[2.56.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.9...v2.56.10\n\n[2.56.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.8...v2.56.9\n\n[2.56.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.7...v2.56.8\n\n[2.56.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.6...v2.56.7\n\n[2.56.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.5...v2.56.6\n\n[2.56.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.4...v2.56.5\n\n[2.56.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.3...v2.56.4\n\n[2.56.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.2...v2.56.3\n\n[2.56.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.1...v2.56.2\n\n[2.56.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.0...v2.56.1\n\n[2.56.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.55.4...v2.56.0\n\n[2.55.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.55.3...v2.55.4\n\n[2.55.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.55.2...v2.55.3\n\n[2.55.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.55.1...v2.55.2\n\n[2.55.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.55.0...v2.55.1\n\n[2.55.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.54.3...v2.55.0\n\n[2.54.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.54.2...v2.54.3\n\n[2.54.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.54.1...v2.54.2\n\n[2.54.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.54.0...v2.54.1\n\n[2.54.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.53.2...v2.54.0\n\n[2.53.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.53.1...v2.53.2\n\n[2.53.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.53.0...v2.53.1\n\n[2.53.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.8...v2.53.0\n\n[2.52.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.7...v2.52.8\n\n[2.52.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.6...v2.52.7\n\n[2.52.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.5...v2.52.6\n\n[2.52.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.4...v2.52.5\n\n[2.52.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.3...v2.52.4\n\n[2.52.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.2...v2.52.3\n\n[2.52.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.1...v2.52.2\n\n[2.52.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.0...v2.52.1\n\n[2.52.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.51.3...v2.52.0\n\n[2.51.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.51.2...v2.51.3\n\n[2.51.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.51.1...v2.51.2\n\n[2.51.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.51.0...v2.51.1\n\n[2.51.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.10...v2.51.0\n\n[2.50.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.9...v2.50.10\n\n[2.50.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.8...v2.50.9\n\n[2.50.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.7...v2.50.8\n\n[2.50.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.6...v2.50.7\n\n[2.50.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.5...v2.50.6\n\n[2.50.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.4...v2.50.5\n\n[2.50.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.3...v2.50.4\n\n[2.50.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.2...v2.50.3\n\n[2.50.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.1...v2.50.2\n\n[2.50.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.0...v2.50.1\n\n[2.50.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.50...v2.50.0\n\n[2.49.50]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.49...v2.49.50\n\n[2.49.49]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.48...v2.49.49\n\n[2.49.48]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.47...v2.49.48\n\n[2.49.47]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.46...v2.49.47\n\n[2.49.46]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.45...v2.49.46\n\n[2.49.45]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.44...v2.49.45\n\n[2.49.44]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.43...v2.49.44\n\n[2.49.43]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.42...v2.49.43\n\n[2.49.42]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.41...v2.49.42\n\n[2.49.41]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.40...v2.49.41\n\n[2.49.40]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.39...v2.49.40\n\n[2.49.39]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.38...v2.49.39\n\n[2.49.38]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.37...v2.49.38\n\n[2.49.37]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.36...v2.49.37\n\n[2.49.36]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.35...v2.49.36\n\n[2.49.35]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.34...v2.49.35\n\n[2.49.34]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.33...v2.49.34\n\n[2.49.33]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.32...v2.49.33\n\n[2.49.32]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.31...v2.49.32\n\n[2.49.31]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.30...v2.49.31\n\n[2.49.30]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.29...v2.49.30\n\n[2.49.29]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.28...v2.49.29\n\n[2.49.28]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.27...v2.49.28\n\n[2.49.27]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.26...v2.49.27\n\n[2.49.26]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.25...v2.49.26\n\n[2.49.25]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.24...v2.49.25\n\n[2.49.24]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.23...v2.49.24\n\n[2.49.23]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.22...v2.49.23\n\n[2.49.22]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.21...v2.49.22\n\n[2.49.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.20...v2.49.21\n\n[2.49.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.19...v2.49.20\n\n[2.49.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.18...v2.49.19\n\n[2.49.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.17...v2.49.18\n\n[2.49.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.16...v2.49.17\n\n[2.49.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.15...v2.49.16\n\n[2.49.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.14...v2.49.15\n\n[2.49.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.13...v2.49.14\n\n[2.49.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.12...v2.49.13\n\n[2.49.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.11...v2.49.12\n\n[2.49.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.10...v2.49.11\n\n[2.49.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.9...v2.49.10\n\n[2.49.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.8...v2.49.9\n\n[2.49.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.7...v2.49.8\n\n[2.49.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.6...v2.49.7\n\n[2.49.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.5...v2.49.6\n\n[2.49.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.4...v2.49.5\n\n[2.49.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.3...v2.49.4\n\n[2.49.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.2...v2.49.3\n\n[2.49.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.1...v2.49.2\n\n[2.49.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.0...v2.49.1\n\n[2.49.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.22...v2.49.0\n\n[2.48.22]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.21...v2.48.22\n\n[2.48.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.20...v2.48.21\n\n[2.48.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.19...v2.48.20\n\n[2.48.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.18...v2.48.19\n\n[2.48.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.17...v2.48.18\n\n[2.48.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.16...v2.48.17\n\n[2.48.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.15...v2.48.16\n\n[2.48.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.14...v2.48.15\n\n[2.48.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.13...v2.48.14\n\n[2.48.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.12...v2.48.13\n\n[2.48.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.11...v2.48.12\n\n[2.48.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.10...v2.48.11\n\n[2.48.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.9...v2.48.10\n\n[2.48.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.8...v2.48.9\n\n[2.48.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.7...v2.48.8\n\n[2.48.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.6...v2.48.7\n\n[2.48.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.5...v2.48.6\n\n[2.48.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.4...v2.48.5\n\n[2.48.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.3...v2.48.4\n\n[2.48.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.2...v2.48.3\n\n[2.48.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.1...v2.48.2\n\n[2.48.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.0...v2.48.1\n\n[2.48.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.32...v2.48.0\n\n[2.47.32]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.31...v2.47.32\n\n[2.47.31]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.30...v2.47.31\n\n[2.47.30]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.29...v2.47.30\n\n[2.47.29]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.28...v2.47.29\n\n[2.47.28]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.27...v2.47.28\n\n[2.47.27]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.26...v2.47.27\n\n[2.47.26]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.25...v2.47.26\n\n[2.47.25]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.24...v2.47.25\n\n[2.47.24]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.23...v2.47.24\n\n[2.47.23]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.22...v2.47.23\n\n[2.47.22]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.21...v2.47.22\n\n[2.47.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.20...v2.47.21\n\n[2.47.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.19...v2.47.20\n\n[2.47.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.18...v2.47.19\n\n[2.47.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.17...v2.47.18\n\n[2.47.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.16...v2.47.17\n\n[2.47.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.15...v2.47.16\n\n[2.47.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.14...v2.47.15\n\n[2.47.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.13...v2.47.14\n\n[2.47.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.12...v2.47.13\n\n[2.47.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.11...v2.47.12\n\n[2.47.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.10...v2.47.11\n\n[2.47.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.9...v2.47.10\n\n[2.47.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.8...v2.47.9\n\n[2.47.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.7...v2.47.8\n\n[2.47.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.6...v2.47.7\n\n[2.47.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.5...v2.47.6\n\n[2.47.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.4...v2.47.5\n\n[2.47.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.3...v2.47.4\n\n[2.47.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.2...v2.47.3\n\n[2.47.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.1...v2.47.2\n\n[2.47.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.0...v2.47.1\n\n[2.47.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.20...v2.47.0\n\n[2.46.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.19...v2.46.20\n\n[2.46.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.18...v2.46.19\n\n[2.46.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.17...v2.46.18\n\n[2.46.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.16...v2.46.17\n\n[2.46.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.15...v2.46.16\n\n[2.46.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.14...v2.46.15\n\n[2.46.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.13...v2.46.14\n\n[2.46.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.12...v2.46.13\n\n[2.46.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.11...v2.46.12\n\n[2.46.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.10...v2.46.11\n\n[2.46.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.9...v2.46.10\n\n[2.46.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.8...v2.46.9\n\n[2.46.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.7...v2.46.8\n\n[2.46.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.6...v2.46.7\n\n[2.46.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.5...v2.46.6\n\n[2.46.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.4...v2.46.5\n\n[2.46.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.3...v2.46.4\n\n[2.46.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.2...v2.46.3\n\n[2.46.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.1...v2.46.2\n\n[2.46.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.0...v2.46.1\n\n[2.46.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.15...v2.46.0\n\n[2.45.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.14...v2.45.15\n\n[2.45.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.13...v2.45.14\n\n[2.45.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.12...v2.45.13\n\n[2.45.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.11...v2.45.12\n\n[2.45.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.10...v2.45.11\n\n[2.45.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.9...v2.45.10\n\n[2.45.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.8...v2.45.9\n\n[2.45.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.7...v2.45.8\n\n[2.45.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.6...v2.45.7\n\n[2.45.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.5...v2.45.6\n\n[2.45.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.4...v2.45.5\n\n[2.45.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.3...v2.45.4\n\n[2.45.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.2...v2.45.3\n\n[2.45.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.1...v2.45.2\n\n[2.45.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.0...v2.45.1\n\n[2.45.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.72...v2.45.0\n\n[2.44.72]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.71...v2.44.72\n\n[2.44.71]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.70...v2.44.71\n\n[2.44.70]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.69...v2.44.70\n\n[2.44.69]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.68...v2.44.69\n\n[2.44.68]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.67...v2.44.68\n\n[2.44.67]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.66...v2.44.67\n\n[2.44.66]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.65...v2.44.66\n\n[2.44.65]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.64...v2.44.65\n\n[2.44.64]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.63...v2.44.64\n\n[2.44.63]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.62...v2.44.63\n\n[2.44.62]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.61...v2.44.62\n\n[2.44.61]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.60...v2.44.61\n\n[2.44.60]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.59...v2.44.60\n\n[2.44.59]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.58...v2.44.59\n\n[2.44.58]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.57...v2.44.58\n\n[2.44.57]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.56...v2.44.57\n\n[2.44.56]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.55...v2.44.56\n\n[2.44.55]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.54...v2.44.55\n\n[2.44.54]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.53...v2.44.54\n\n[2.44.53]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.52...v2.44.53\n\n[2.44.52]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.51...v2.44.52\n\n[2.44.51]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.50...v2.44.51\n\n[2.44.50]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.49...v2.44.50\n\n[2.44.49]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.48...v2.44.49\n\n[2.44.48]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.47...v2.44.48\n\n[2.44.47]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.46...v2.44.47\n\n[2.44.46]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.45...v2.44.46\n\n[2.44.45]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.44...v2.44.45\n\n[2.44.44]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.43...v2.44.44\n\n[2.44.43]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.42...v2.44.43\n\n[2.44.42]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.41...v2.44.42\n\n[2.44.41]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.40...v2.44.41\n\n[2.44.40]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.39...v2.44.40\n\n[2.44.39]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.38...v2.44.39\n\n[2.44.38]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.37...v2.44.38\n\n[2.44.37]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.36...v2.44.37\n\n[2.44.36]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.35...v2.44.36\n\n[2.44.35]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.34...v2.44.35\n\n[2.44.34]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.33...v2.44.34\n\n[2.44.33]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.32...v2.44.33\n\n[2.44.32]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.31...v2.44.32\n\n[2.44.31]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.30...v2.44.31\n\n[2.44.30]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.29...v2.44.30\n\n[2.44.29]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.28...v2.44.29\n\n[2.44.28]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.27...v2.44.28\n\n[2.44.27]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.26...v2.44.27\n\n[2.44.26]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.25...v2.44.26\n\n[2.44.25]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.24...v2.44.25\n\n[2.44.24]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.23...v2.44.24\n\n[2.44.23]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.22...v2.44.23\n\n[2.44.22]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.21...v2.44.22\n\n[2.44.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.20...v2.44.21\n\n[2.44.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.19...v2.44.20\n\n[2.44.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.18...v2.44.19\n\n[2.44.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.17...v2.44.18\n\n[2.44.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.16...v2.44.17\n\n[2.44.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.15...v2.44.16\n\n[2.44.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.14...v2.44.15\n\n[2.44.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.13...v2.44.14\n\n[2.44.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.12...v2.44.13\n\n[2.44.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.11...v2.44.12\n\n[2.44.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.10...v2.44.11\n\n[2.44.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.9...v2.44.10\n\n[2.44.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.8...v2.44.9\n\n[2.44.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.7...v2.44.8\n\n[2.44.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.6...v2.44.7\n\n[2.44.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.5...v2.44.6\n\n[2.44.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.4...v2.44.5\n\n[2.44.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.3...v2.44.4\n\n[2.44.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.2...v2.44.3\n\n[2.44.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.1...v2.44.2\n\n[2.44.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.0...v2.44.1\n\n[2.44.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.43.7...v2.44.0\n\n[2.43.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.43.6...v2.43.7\n\n[2.43.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.43.5...v2.43.6\n\n[2.43.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.43.4...v2.43.5\n\n[2.43.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.43.3...v2.43.4\n\n[2.43.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.43.2...v2.43.3\n\n[2.43.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.43.1...v2.43.2\n\n[2.43.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.43.0...v2.43.1\n\n[2.43.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.42...v2.43.0\n\n[2.42.42]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.41...v2.42.42\n\n[2.42.41]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.40...v2.42.41\n\n[2.42.40]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.39...v2.42.40\n\n[2.42.39]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.38...v2.42.39\n\n[2.42.38]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.37...v2.42.38\n\n[2.42.37]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.36...v2.42.37\n\n[2.42.36]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.35...v2.42.36\n\n[2.42.35]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.34...v2.42.35\n\n[2.42.34]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.33...v2.42.34\n\n[2.42.33]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.32...v2.42.33\n\n[2.42.32]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.31...v2.42.32\n\n[2.42.31]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.30...v2.42.31\n\n[2.42.30]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.29...v2.42.30\n\n[2.42.29]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.28...v2.42.29\n\n[2.42.28]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.27...v2.42.28\n\n[2.42.27]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.26...v2.42.27\n\n[2.42.26]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.25...v2.42.26\n\n[2.42.25]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.24...v2.42.25\n\n[2.42.24]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.23...v2.42.24\n\n[2.42.23]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.22...v2.42.23\n\n[2.42.22]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.21...v2.42.22\n\n[2.42.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.20...v2.42.21\n\n[2.42.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.19...v2.42.20\n\n[2.42.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.18...v2.42.19\n\n[2.42.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.17...v2.42.18\n\n[2.42.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.16...v2.42.17\n\n[2.42.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.15...v2.42.16\n\n[2.42.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.14...v2.42.15\n\n[2.42.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.13...v2.42.14\n\n[2.42.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.12...v2.42.13\n\n[2.42.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.11...v2.42.12\n\n[2.42.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.10...v2.42.11\n\n[2.42.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.9...v2.42.10\n\n[2.42.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.8...v2.42.9\n\n[2.42.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.7...v2.42.8\n\n[2.42.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.6...v2.42.7\n\n[2.42.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.5...v2.42.6\n\n[2.42.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.4...v2.42.5\n\n[2.42.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.3...v2.42.4\n\n[2.42.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.2...v2.42.3\n\n[2.42.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.1...v2.42.2\n\n[2.42.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.0...v2.42.1\n\n[2.42.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.18...v2.42.0\n\n[2.41.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.17...v2.41.18\n\n[2.41.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.16...v2.41.17\n\n[2.41.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.15...v2.41.16\n\n[2.41.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.14...v2.41.15\n\n[2.41.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.13...v2.41.14\n\n[2.41.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.12...v2.41.13\n\n[2.41.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.11...v2.41.12\n\n[2.41.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.10...v2.41.11\n\n[2.41.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.9...v2.41.10\n\n[2.41.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.8...v2.41.9\n\n[2.41.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.7...v2.41.8\n\n[2.41.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.6...v2.41.7\n\n[2.41.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.5...v2.41.6\n\n[2.41.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.4...v2.41.5\n\n[2.41.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.3...v2.41.4\n\n[2.41.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.2...v2.41.3\n\n[2.41.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.1...v2.41.2\n\n[2.41.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.0...v2.41.1\n\n[2.41.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.40.2...v2.41.0\n\n[2.40.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.40.1...v2.40.2\n\n[2.40.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.40.0...v2.40.1\n\n[2.40.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.39.2...v2.40.0\n\n[2.39.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.39.1...v2.39.2\n\n[2.39.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.39.0...v2.39.1\n\n[2.39.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.38.7...v2.39.0\n\n[2.38.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.38.6...v2.38.7\n\n[2.38.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.38.5...v2.38.6\n\n[2.38.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.38.4...v2.38.5\n\n[2.38.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.38.3...v2.38.4\n\n[2.38.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.38.2...v2.38.3\n\n[2.38.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.38.1...v2.38.2\n\n[2.38.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.38.0...v2.38.1\n\n[2.38.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.37.0...v2.38.0\n\n[2.37.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.36.0...v2.37.0\n\n[2.36.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.35.0...v2.36.0\n\n[2.35.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.34.3...v2.35.0\n\n[2.34.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.34.2...v2.34.3\n\n[2.34.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.34.1...v2.34.2\n\n[2.34.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.34.0...v2.34.1\n\n[2.34.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.36...v2.34.0\n\n[2.33.36]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.35...v2.33.36\n\n[2.33.35]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.34...v2.33.35\n\n[2.33.34]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.33...v2.33.34\n\n[2.33.33]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.32...v2.33.33\n\n[2.33.32]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.31...v2.33.32\n\n[2.33.31]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.30...v2.33.31\n\n[2.33.30]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.29...v2.33.30\n\n[2.33.29]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.28...v2.33.29\n\n[2.33.28]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.27...v2.33.28\n\n[2.33.27]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.26...v2.33.27\n\n[2.33.26]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.25...v2.33.26\n\n[2.33.25]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.24...v2.33.25\n\n[2.33.24]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.23...v2.33.24\n\n[2.33.23]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.22...v2.33.23\n\n[2.33.22]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.21...v2.33.22\n\n[2.33.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.20...v2.33.21\n\n[2.33.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.19...v2.33.20\n\n[2.33.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.18...v2.33.19\n\n[2.33.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.17...v2.33.18\n\n[2.33.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.16...v2.33.17\n\n[2.33.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.15...v2.33.16\n\n[2.33.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.14...v2.33.15\n\n[2.33.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.13...v2.33.14\n\n[2.33.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.12...v2.33.13\n\n[2.33.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.11...v2.33.12\n\n[2.33.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.10...v2.33.11\n\n[2.33.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.9...v2.33.10\n\n[2.33.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.8...v2.33.9\n\n[2.33.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.7...v2.33.8\n\n[2.33.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.6...v2.33.7\n\n[2.33.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.5...v2.33.6\n\n[2.33.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.4...v2.33.5\n\n[2.33.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.3...v2.33.4\n\n[2.33.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.2...v2.33.3\n\n[2.33.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.1...v2.33.2\n\n[2.33.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.0...v2.33.1\n\n[2.33.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.20...v2.33.0\n\n[2.32.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.19...v2.32.20\n\n[2.32.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.18...v2.32.19\n\n[2.32.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.17...v2.32.18\n\n[2.32.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.16...v2.32.17\n\n[2.32.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.15...v2.32.16\n\n[2.32.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.14...v2.32.15\n\n[2.32.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.13...v2.32.14\n\n[2.32.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.12...v2.32.13\n\n[2.32.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.11...v2.32.12\n\n[2.32.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.10...v2.32.11\n\n[2.32.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.9...v2.32.10\n\n[2.32.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.8...v2.32.9\n\n[2.32.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.7...v2.32.8\n\n[2.32.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.6...v2.32.7\n\n[2.32.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.5...v2.32.6\n\n[2.32.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.4...v2.32.5\n\n[2.32.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.3...v2.32.4\n\n[2.32.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.2...v2.32.3\n\n[2.32.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.1...v2.32.2\n\n[2.32.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.0...v2.32.1\n\n[2.32.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.31.3...v2.32.0\n\n[2.31.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.31.2...v2.31.3\n\n[2.31.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.31.1...v2.31.2\n\n[2.31.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.31.0...v2.31.1\n\n[2.31.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.30.0...v2.31.0\n\n[2.30.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.29.8...v2.30.0\n\n[2.29.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.29.7...v2.29.8\n\n[2.29.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.29.6...v2.29.7\n\n[2.29.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.29.5...v2.29.6\n\n[2.29.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.29.4...v2.29.5\n\n[2.29.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.29.3...v2.29.4\n\n[2.29.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.29.2...v2.29.3\n\n[2.29.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.29.1...v2.29.2\n\n[2.29.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.29.0...v2.29.1\n\n[2.29.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.16...v2.29.0\n\n[2.28.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.15...v2.28.16\n\n[2.28.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.14...v2.28.15\n\n[2.28.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.13...v2.28.14\n\n[2.28.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.12...v2.28.13\n\n[2.28.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.11...v2.28.12\n\n[2.28.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.10...v2.28.11\n\n[2.28.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.9...v2.28.10\n\n[2.28.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.8...v2.28.9\n\n[2.28.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.7...v2.28.8\n\n[2.28.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.6...v2.28.7\n\n[2.28.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.5...v2.28.6\n\n[2.28.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.4...v2.28.5\n\n[2.28.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.3...v2.28.4\n\n[2.28.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.2...v2.28.3\n\n[2.28.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.1...v2.28.2\n\n[2.28.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.0...v2.28.1\n\n[2.28.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.15...v2.28.0\n\n[2.27.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.14...v2.27.15\n\n[2.27.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.13...v2.27.14\n\n[2.27.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.12...v2.27.13\n\n[2.27.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.11...v2.27.12\n\n[2.27.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.10...v2.27.11\n\n[2.27.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.9...v2.27.10\n\n[2.27.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.8...v2.27.9\n\n[2.27.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.7...v2.27.8\n\n[2.27.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.6...v2.27.7\n\n[2.27.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.5...v2.27.6\n\n[2.27.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.4...v2.27.5\n\n[2.27.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.3...v2.27.4\n\n[2.27.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.2...v2.27.3\n\n[2.27.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.1...v2.27.2\n\n[2.27.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.0...v2.27.1\n\n[2.27.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.20...v2.27.0\n\n[2.26.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.19...v2.26.20\n\n[2.26.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.18...v2.26.19\n\n[2.26.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.17...v2.26.18\n\n[2.26.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.16...v2.26.17\n\n[2.26.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.15...v2.26.16\n\n[2.26.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.14...v2.26.15\n\n[2.26.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.13...v2.26.14\n\n[2.26.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.12...v2.26.13\n\n[2.26.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.11...v2.26.12\n\n[2.26.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.10...v2.26.11\n\n[2.26.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.9...v2.26.10\n\n[2.26.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.8...v2.26.9\n\n[2.26.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.7...v2.26.8\n\n[2.26.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.6...v2.26.7\n\n[2.26.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.5...v2.26.6\n\n[2.26.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.4...v2.26.5\n\n[2.26.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.3...v2.26.4\n\n[2.26.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.2...v2.26.3\n\n[2.26.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.1...v2.26.2\n\n[2.26.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.0...v2.26.1\n\n[2.26.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.25.11...v2.26.0\n\n[2.25.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.25.10...v2.25.11\n\n[2.25.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.25.9...v2.25.10\n\n[2.25.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.25.8...v2.25.9\n\n[2.25.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.25.7...v2.25.8\n\n[2.25.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.25.6...v2.25.7\n\n[2.25.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.25.5...v2.25.6\n\n[2.25.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.25.4...v2.25.5\n\n[2.25.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.25.3...v2.25.4\n\n[2.25.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.25.2...v2.25.3\n\n[2.25.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.25.1...v2.25.2\n\n[2.25.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.25.0...v2.25.1\n\n[2.25.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.24.4...v2.25.0\n\n[2.24.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.24.3...v2.24.4\n\n[2.24.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.24.2...v2.24.3\n\n[2.24.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.24.1...v2.24.2\n\n[2.24.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.24.0...v2.24.1\n\n[2.24.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.23.9...v2.24.0\n\n[2.23.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.23.8...v2.23.9\n\n[2.23.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.23.7...v2.23.8\n\n[2.23.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.23.6...v2.23.7\n\n[2.23.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.23.5...v2.23.6\n\n[2.23.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.23.4...v2.23.5\n\n[2.23.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.23.3...v2.23.4\n\n[2.23.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.23.2...v2.23.3\n\n[2.23.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.23.1...v2.23.2\n\n[2.23.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.23.0...v2.23.1\n\n[2.23.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.22.10...v2.23.0\n\n[2.22.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.22.9...v2.22.10\n\n[2.22.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.22.8...v2.22.9\n\n[2.22.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.22.7...v2.22.8\n\n[2.22.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.22.6...v2.22.7\n\n[2.22.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.22.5...v2.22.6\n\n[2.22.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.22.4...v2.22.5\n\n[2.22.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.22.3...v2.22.4\n\n[2.22.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.22.2...v2.22.3\n\n[2.22.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.22.1...v2.22.2\n\n[2.22.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.22.0...v2.22.1\n\n[2.22.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.21.27...v2.22.0\n\n[2.21.27]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.21.26...v2.21.27\n\n[2.21.26]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.21.25...v2.21.26\n\n[2.21.25]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.21.24...v2.21.25\n\n[2.21.24]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.21.23...v2.21.24\n\n[2.21.23]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.21.22...v2.21.23\n\n[2.21.22]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.21.21...v2.21.22\n\n[2.21.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.21.20...v2.21.21\n\n[2.21.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.21.19...v2.21.20\n\n[2.21.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.21.18...v2.21.19\n\n[2.21.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.21.17...v2.21.18\n\n[2.21.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.21.16...v2.21.17\n\n[2.21.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.21.15...v2.21.16\n\n[2.21.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.21.14...v2.21.\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - \"before 8am on Monday\" (UTC),\nAutomerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n👻 **Immortal**: This PR will be recreated if closed unmerged. Get\n[config\nhelp](https://redirect.github.com/renovatebot/renovate/discussions) if\nthat's undesired.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0MS44Mi43IiwidXBkYXRlZEluVmVyIjoiNDEuODIuNyIsInRhcmdldEJyYW5jaCI6Im1haW4iLCJsYWJlbHMiOlsiZGVwZW5kZW5jaWVzIl19-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2025-08-26T00:40:43Z",
          "tree_id": "809f60cfd35c8ebf077994c05daa91de53a23768",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/110b3e365c85ae517ab944c9d93de5d9716a4d3e"
        },
        "date": 1756170063803,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 742500,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 22275000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 22275000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-avg",
            "value": 5.85,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 6.93,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 170.82,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 205.11,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 736500,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 22095000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 22095000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 5.7,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 6.79,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 127.99,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 150.43,
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
          "id": "bb2da1138065e8c67ca23a8f0b08c46d5e7cdfe3",
          "message": "Upgrade collector dependencies to v0.133.0/v1.39.0 (#1010)\n\nNotable upgrade, this also bumps minimum Go version from `1.23.0` to\n`1.24`, see:\n- [collector\n#13627](https://github.com/open-telemetry/opentelemetry-collector/pull/13627)\n- [collector-contrib\n#41968](https://github.com/open-telemetry/opentelemetry-collector-contrib/pull/41968).\n\n(Also updated CHANGELOG for a few recent changes)",
          "timestamp": "2025-08-26T16:46:36Z",
          "tree_id": "c6cad1cf7fd1883148d2cd44b8bda265f3bf85f3",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/bb2da1138065e8c67ca23a8f0b08c46d5e7cdfe3"
        },
        "date": 1756228877317,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 730333.3333333334,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 21910000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 21910000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-avg",
            "value": 5.69,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 6.75,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 161.56,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 193.11,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 732333.3333333334,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 21970000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 21970000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 5.62,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 6.66,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 126.45,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 145.95,
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
          "id": "d36e3843a8abf5eeaea1b2fd29c31069857cdd97",
          "message": "Add periodic flush to parquet exporter using timer tick (#1004)\n\nFixes: #499 \n\nUp until this point, the only way that parquet exporter would flush &\nclose the writers (e.g. write the completed file), would be if the file\nreaches some configured max threshold or if the pipeline shut down. This\nis not great for users who may wish to have their data visible sooner,\nespecially if it takes some time to accumulate enough rows.\n\nThis PR uses the newly implemented TimerTick mechanism to periodically\nflush any files that are older than some configured threshold.\n\n---------\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>\nCo-authored-by: Laurent Quérel <laurent.querel@gmail.com>",
          "timestamp": "2025-08-26T18:10:32Z",
          "tree_id": "d36cf7542441f72821e0e83d8969440b0e437a64",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/d36e3843a8abf5eeaea1b2fd29c31069857cdd97"
        },
        "date": 1756232387955,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 738666.6666666666,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 22160000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 22160000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-avg",
            "value": 5.66,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 6.59,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 157.09,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 184.2,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 739500,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 22185000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 22185000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 5.69,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 6.75,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 125.13,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 152.2,
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
          "id": "731db26792a3bbbfb40db9b20cb9ac785fd093e1",
          "message": "[query-engine] Static resolution improvements (#1007)\n\n## Changes\n\n* Simplify the signature for static resolution methods (less lifetimes)\n* Implement some static folding when static resolution is manually\ninvoked by parsers\n\n---------\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2025-08-26T20:22:58Z",
          "tree_id": "fa6820c7393b5f92b122c876ce25a4e148c0eb0d",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/731db26792a3bbbfb40db9b20cb9ac785fd093e1"
        },
        "date": 1756240242959,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 742000,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 22260000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 22260000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-avg",
            "value": 5.6,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 6.76,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 164.54,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 191.19,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 743000,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 22290000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 22290000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 5.69,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 6.74,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 125.69,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 147.67,
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
          "id": "9fce232f35d3222fa3ec8fb648e06928345cc0d6",
          "message": "fix the parquet shutdown test flakiness again (#1011)\n\nHopefully this will finally fix the parquet shutdown test being flakey 🤞\n\nInstead of buffering up enough rows and _hoping_ that when we send the\nshutdown signal that the writer won't have enough time to flush them, we\nuse an implementation of object store that forces there to be a delay\nbefore we write to the filesystem. That way when we call shutdown, the\nparquet should call flush_all on the writer manager, which will close\nthe parquet file writers, which will hit this delay when they try to\nwrite the parquet footer, which will _hopefully_ cause a timeout and\nreliably this test will pass..",
          "timestamp": "2025-08-27T00:18:16Z",
          "tree_id": "9eee9cee823bafdf180146f0d87fca0c112538e9",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/9fce232f35d3222fa3ec8fb648e06928345cc0d6"
        },
        "date": 1756254403265,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 742333.3333333334,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 22270000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 22270000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-avg",
            "value": 5.8,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 6.8,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 164.8,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 195.82,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 733333.3333333334,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 22000000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 22000000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 5.51,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 6.48,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 123.62,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 143.08,
            "unit": "MiB"
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
          "distinct": false,
          "id": "9859c859192f45e865c38fde9e616407e76c198b",
          "message": "Generic, high-performance, type-safe metric instrumentation framework (#946)\n\nThis instrumentation framework is declarative (macro-style annotations),\nsupports multivariate metrics, is compatible with our thread-per-core\ndesign, offers low overhead, and is NUMA-compatible (not yet fully\nimplemented).\n\nThis framework will be integrated with the OTEL Rust Client SDK in an\nupcoming PR. Currently, 3 HTTP endpoints are exposed to facilitate\nintegration with our benchmark infrastructure:\n\n- `/telemetry/live-schema`: current semantic conventions registry\n- `/telemetry/metrics`: current aggregated metrics in JSON, line\nprotocol, or\nPrometheus text format. Supported parameters: reset=true,false,\nformat=json (default), json_compact, line_protocol, prometheus.\n- `/telemetry/metrics/aggregate`: aggregated metrics grouped by metric\nset name\nand optional attributes. Supported parameters: reset, format,\nattrs=comma separated list of attribute names to group by.\n\n<img width=\"1537\" height=\"830\" alt=\"image\"\nsrc=\"https://github.com/user-attachments/assets/ec4647ad-ca58-4314-a954-9424733cfe3f\"\n/>\n\n## How to define a metric set\n\n- Import instrument types from otap-df-telemetry and the macro from this\ncrate.\n- Annotate your struct with `#[metric_set(name =\n\"<metrics.group.name>\")]`.\n- For each metric field, choose one of the supported instruments and add\n  `#[metric(unit = \"{unit}\")]`.\n- Supported instruments: `Counter<u64>` (`UpDownCounter<u64>`,\n`Gauge<u64>`\n    soon).\n  - Units follow a simple string convention (e.g., `{msg}`, `{record}`,\n    `{span}`).\n- Optional: Document each field with a Rust doc comment; it becomes the\nmetric\n  \"brief\" in the descriptor.\n- Optional: Override a field metric name with\n  `#[metric(name = \"custom.name\", unit = \"{unit}\")]`.\n- If `name` is omitted, the field identifier is converted by replacing\n`_`\n    with `.`.\n\nExample (from the OTAP Perf Exporter):\n\n```rust\nuse otap_df_telemetry::instrument::{Counter, UpDownCounter, Gauge};\nuse otap_df_telemetry_macros::metric_set;\n\n/// Pdata-oriented metrics for the OTAP PerfExporter.\n#[metric_set(name = \"perf.exporter.pdata.metrics\")]\n#[derive(Debug, Default, Clone)]\npub struct PerfExporterPdataMetrics {\n    /// Number of pdata batches received.\n    #[metric(unit = \"{msg}\")]\n    pub batches: Counter<u64>,\n\n    /// Number of invalid pdata batches received.\n    #[metric(unit = \"{msg}\")]\n    pub invalid_batches: Counter<u64>,\n\n    /// Number of Arrow records received.\n    #[metric(unit = \"{record}\")]\n    pub arrow_records: Counter<u64>,\n\n    /// Number of logs received.\n    #[metric(unit = \"{log}\")]\n    pub logs: Counter<u64>,\n\n    /// Number of spans received.\n    #[metric(unit = \"{span}\")]\n    pub spans: Counter<u64>,\n\n    /// Number of metrics received.\n    #[metric(unit = \"{metric}\")]\n    pub metrics: Counter<u64>,\n}\n```\n\n## Predefined Attributes\n\nThe pipeline engine defines a set of predefined attributes that can be\nused for\nlabeling metrics and traces. These attributes provide context about the\npipeline\nand its components, facilitating better observability and analysis.\n\n| Scope | Attribute | Type | Description |\n\n|----------|---------------------|---------|--------------------------------------------------------------|\n| Resource | process_instance_id | string | Unique process instance\nidentifier (base32-encoded UUID v7). |\n| Resource | host_id | string | Host identifier (e.g. hostname). |\n| Resource | container_id | string | Container identifier (e.g.\nDocker/containerd container ID). |\n| Engine | core_id | integer | Core identifier. |\n| Engine | numa_node_id | integer | NUMA node identifier. |\n| Pipeline | pipeline_id | string | Pipeline identifier. |\n| Node | node_id | string | Node unique identifier (in scope of the\npipeline). |\n| Node | node_type | string | Node type (e.g. \"receiver\", \"processor\",\n\"exporter\"). |\n\n\nCloses https://github.com/open-telemetry/otel-arrow/issues/833\nCloses https://github.com/open-telemetry/otel-arrow/issues/986\n\n---------\n\nCo-authored-by: Utkarsh Umesan Pillai <66651184+utpilla@users.noreply.github.com>\nCo-authored-by: Joshua MacDonald <jmacd@users.noreply.github.com>",
          "timestamp": "2025-08-27T07:10:17Z",
          "tree_id": "33545484b093eeacb94bb8344d9a504cbc0e9c25",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/9859c859192f45e865c38fde9e616407e76c198b"
        },
        "date": 1756279148592,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 741000,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 22230000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 22230000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-avg",
            "value": 5.7,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 6.75,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 166.04,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 193.46,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 725666.6666666666,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 21770000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 21770000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 5.58,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 6.54,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 122.7,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 140.29,
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
          "id": "67f3d7bdba4309ee309dcafe9584aa740509c564",
          "message": "[query-engine] Concat & Join expressions (#1014)\n\n## Changes\n\n* Adds array concat, string concat, and string join expressions +\nrecordset implementation\n* Adds KQL support for `strcat`, `strcat_delim`, & `array_concat`",
          "timestamp": "2025-08-27T17:30:10Z",
          "tree_id": "726986c89ddb0c4defeea05bd4dcd1a271440126",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/67f3d7bdba4309ee309dcafe9584aa740509c564"
        },
        "date": 1756316315004,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 738500,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 22155000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 22155000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-avg",
            "value": 5.64,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 6.75,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 168.53,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 198.41,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 734333.3333333334,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 22030000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 22030000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 5.69,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 6.94,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 129.82,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 153.99,
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
          "id": "8bcc94d58de8778ac28700ec2f760a284f2c70c5",
          "message": "Upgrade to arrow 56.1 to improve adaptive schema handling in parquet exporter (#1006)\n\npart of #863 \n\nImprovements were made in the latest version of arrow-rs\n(https://github.com/apache/arrow-rs/pull/8095 and\nhttps://github.com/apache/arrow-rs/pull/8005) to improve parquet\nwriter's ability to handle logically equivalent data types for the same\ncolumn.\n\nWe need this b/c different OTAP batches might switch between Dictionary\nencoding (with various key sizes) and native encoding for the same\nrecord batch. The latest version of parquet is able to accept these\ndifferent array types for the same column.\n\nA test is added to ensure the behaviour works as expected.\n\n---------\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2025-08-27T18:10:46Z",
          "tree_id": "a6c4a940ffebde286418afc84953fe7bc5db7067",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/8bcc94d58de8778ac28700ec2f760a284f2c70c5"
        },
        "date": 1756319345388,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 735833.3333333334,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 22075000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 22075000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 5.52,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 6.6,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 120.99,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 142.05,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 740500,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 22215000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 22215000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-avg",
            "value": 5.73,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 6.93,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 170.11,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 198.66,
            "unit": "MiB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "197425009+otelbot[bot]@users.noreply.github.com",
            "name": "otelbot[bot]",
            "username": "otelbot[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "8147563ab7d388eeeacbc21c9063c10145bc99aa",
          "message": "chore(release) Prepare Release v0.42.0 (#1016)\n\n## Release v0.42.0\n\nThis PR prepares the repository for release v0.42.0.\n\n### Changes included:\n- Updated CHANGELOG.md with release notes\n- Updated collector/otelarrowcol-build.yaml version to v0.42.0\n- Updated collector/cmd/otelarrowcol/main.go version to v0.42.0\n\n### Release Notes:\n- Standardize to shorthand license header.\n[#954](https://github.com/open-telemetry/otel-arrow/pull/954)\n- Fix logs handling of missing optional trace_id and span_id columns.\n[#973](https://github.com/open-telemetry/otel-arrow/pull/973)\n- Upgrade to v0.133.0 / v1.37.0 of collector dependencies.\n[#890](https://github.com/open-telemetry/otel-arrow/pull/890),\n[#1010](https://github.com/open-telemetry/otel-arrow/pull/1010)\n- Notable upgrade, this also bumps minimum Go version from `1.23.0` to\n`1.24`, see [collector\n#13627](https://github.com/open-telemetry/opentelemetry-collector/pull/13627)\nand [collector-contrib\n#41968](https://github.com/open-telemetry/opentelemetry-collector-contrib/pull/41968).\n\n### Checklist:\n- [x] Verify CHANGELOG.md formatting and content\n- [x] Verify collector version update in\ncollector/otelarrowcol-build.yaml\n- [x] Verify collector main.go version update in\ncollector/cmd/otelarrowcol/main.go\n- [x] Confirm all tests pass\n- [x] Ready to merge and tag release\n\nAfter merging this PR, run the **Push Release** workflow to create git\ntags and publish the GitHub release.\n\nCo-authored-by: otelbot <197425009+otelbot@users.noreply.github.com>",
          "timestamp": "2025-08-27T18:56:32Z",
          "tree_id": "42ca579753b1aafb63d76d2c98eb4e4d02121a7a",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/8147563ab7d388eeeacbc21c9063c10145bc99aa"
        },
        "date": 1756321515705,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 736333.3333333334,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 22090000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 22090000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-avg",
            "value": 5.77,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 6.83,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 164.26,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 195.07,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 738500,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 22155000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 22155000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 5.57,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 6.51,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 121.97,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 143.57,
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
          "id": "8c4473eefcd9f1107e49a56bb26d3b74d4c97ab0",
          "message": "[query-engine] Refactor try_resolve_value_type calls in KQL parser to use new helper method (#1020)\n\nnt",
          "timestamp": "2025-08-27T19:22:03Z",
          "tree_id": "feb6790bea8b24da6bbfe10fbda5876b5cf7cad3",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/8c4473eefcd9f1107e49a56bb26d3b74d4c97ab0"
        },
        "date": 1756323040355,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 735833.3333333334,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 22075000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 22075000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 5.66,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 6.81,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 119.38,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 141.97,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 742333.3333333334,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 22270000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 22270000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-avg",
            "value": 5.68,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 6.75,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 163.99,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 198.62,
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
          "id": "232bf368a14f0b15f2a224004dea5e41fae08148",
          "message": "Script around Github markdown wrapping display differences for Release process (#1019)\n\nWhen using the `Prepare Release` and `Push Release` actions the\nextracted Changelog content displays incorrectly. The repo lint enforces\nline length in markdown files, leading to extra newlines in raw content:\n\n```md\n- Upgrade to v0.133.0 / v1.37.0 of collector dependencies.\n  [#890](https://github.com/open-telemetry/otel-arrow/pull/890),\n  [#1010](https://github.com/open-telemetry/otel-arrow/pull/1010)\n  - Notable upgrade, this also bumps minimum Go version from `1.23.0` to `1.24`,\n    see [collector\n    #13627](https://github.com/open-telemetry/opentelemetry-collector/pull/13627)\n    and [collector-contrib\n    #41968](https://github.com/open-telemetry/opentelemetry-collector-contrib/pull/41968).\n```\n\nWhen viewing the [Changelog\nitself](https://github.com/open-telemetry/otel-arrow/blob/main/CHANGELOG.md)\nGitHub renders this in proper way. However, when we extract it to embed\nin a PR body (in `Prepare Release`) or in a Release object (in `Push\nRelease`) the rendering is not the same:\n\n<img width=\"1078\" height=\"539\" alt=\"image\"\nsrc=\"https://github.com/user-attachments/assets/9eb14da1-4754-4a2c-962b-7bde9267bcba\"\n/>\n\nThe script now normalizes whitespace in the extracted content to display\nproperly in the specific use cases we have:\n\n<img width=\"2045\" height=\"417\" alt=\"image\"\nsrc=\"https://github.com/user-attachments/assets/ca07579e-ca69-4ef3-8954-902a6f0c2852\"\n/>\n\nSmall improvement but saves 2 seconds of manual tweaking during each\nrelease 😄",
          "timestamp": "2025-08-27T21:09:47Z",
          "tree_id": "941287fa7b127f977c3ccf3da6fb44351bc5565f",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/232bf368a14f0b15f2a224004dea5e41fae08148"
        },
        "date": 1756329506525,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 739333.3333333334,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 22180000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 22180000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-avg",
            "value": 5.8,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 6.86,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 162.79,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 191.48,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 736833.3333333334,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 22105000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 22105000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 5.71,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 6.7,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 129.64,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 153.96,
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
          "distinct": false,
          "id": "8bc34732563267c0b2b85036028e429097928e22",
          "message": "Split Error and TypedError<T>; generalize NodeControlMsg<PData> (#1017)\n\nPart of #509 \n\nWhile working to implement the rest of the retry_processor support,\ndiscovered a need to add `<PData>` to the `NodeControlMsg`. The `Nack`\nmessage now includes an optional `pdata` field.\n\nAt the same time, most uses of Error did not require a `<T>`, so this\nseparates them and handles conversion to and from the `TypedError<T>`\nform. The type is erased and the error message is stored as a string.\n\nNo functional changes except w.r.t. error structure and a new\n`Option<Box<PData>>` in the Nack. While these changes are unrelated,\nthey have approximately the same impact and combining them was\nrelatively easy.\n\n---------\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2025-08-27T21:49:19Z",
          "tree_id": "7f41b78ef420559bc1f5b43d2b7731c08c7a53de",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/8bc34732563267c0b2b85036028e429097928e22"
        },
        "date": 1756331851526,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 734166.6666666666,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 22025000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 22025000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 5.71,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 6.76,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 127.14,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 148.45,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 734166.6666666666,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 22025000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 22025000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-avg",
            "value": 5.65,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 6.61,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 162.27,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 193.53,
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
          "id": "c521a9a5e672e2ead3cc21fb1a15c6d9a82b854f",
          "message": "[perf] - modify loadgen threads to update metrics/stats when it exits to reduce contention (#1021)",
          "timestamp": "2025-08-27T21:52:18Z",
          "tree_id": "5f9408be7fc9ffdbaa8dda47b6da08872b9b73b5",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/c521a9a5e672e2ead3cc21fb1a15c6d9a82b854f"
        },
        "date": 1756332050591,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 746833.3333333334,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 22405000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 22405000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 5.78,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 6.67,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 129.6,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 153.22,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 732666.6666666666,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 21980000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 21980000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-avg",
            "value": 5.73,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 6.73,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 164.41,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 192.55,
            "unit": "MiB"
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
          "id": "0d3422f152e8dd7f188bf0c2e9f71b3f4f670e3f",
          "message": "OTLP and OTAP Exporter Metrics (#1023)\n\nThis PR is a first iteration to progressively comply with this [RFC -\nPipeline Component\nTelemetry](https://github.com/open-telemetry/opentelemetry-collector/blob/main/docs/rfcs/component-universal-telemetry.md).\n\nThe OTLP and OTAP exporters have been updated.\n\nThe following metric set has been used:\n```rust\n/// PData metrics for exporters.\n/// Namespace grouped under `exporter.pdata`.\n/// Uses a monotonic counter per outcome.\n#[metric_set(name = \"exporter.pdata\")]\n#[derive(Debug, Default, Clone)]\npub struct ExporterPDataMetrics {\n    /// Number of pdata metrics consumed by this exporter.\n    #[metric(unit = \"{msg}\")]\n    pub metrics_consumed: Counter<u64>,\n\n    /// Number of pdata metrics successfully exported.\n    #[metric(unit = \"{msg}\")]\n    pub metrics_exported: Counter<u64>,\n\n    /// Number of pdata metrics that failed to be exported.\n    #[metric(unit = \"{msg}\")]\n    pub metrics_failed: Counter<u64>,\n\n    /// Number of pdata logs consumed by this exporter.\n    #[metric(unit = \"{msg}\")]\n    pub logs_consumed: Counter<u64>,\n\n    /// Number of pdata logs successfully exported.\n    #[metric(unit = \"{msg}\")]\n    pub logs_exported: Counter<u64>,\n\n    /// Number of pdata logs that failed to be exported.\n    #[metric(unit = \"{msg}\")]\n    pub logs_failed: Counter<u64>,\n\n    /// Number of pdata traces consumed by this exporter.\n    #[metric(unit = \"{msg}\")]\n    pub traces_consumed: Counter<u64>,\n\n    /// Number of pdata traces successfully exported.\n    #[metric(unit = \"{msg}\")]\n    pub traces_exported: Counter<u64>,\n\n    /// Number of pdata traces that failed to be exported.\n    #[metric(unit = \"{msg}\")]\n    pub traces_failed: Counter<u64>,\n}\n```",
          "timestamp": "2025-08-28T20:57:08Z",
          "tree_id": "0ed023f63d412c4d8bf401e5c22b80962dd4e1fc",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/0d3422f152e8dd7f188bf0c2e9f71b3f4f670e3f"
        },
        "date": 1756415137924,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 731666.6666666666,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 21950000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 21950000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 5.61,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 6.59,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 119.42,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 141.46,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 738166.6666666666,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 22145000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 22145000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-avg",
            "value": 5.8,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 6.93,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 161.69,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 187.93,
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
          "id": "fc324c575adcd90158964e2de0725c4a40e32cf2",
          "message": "Parquet exporter handle optional fields (#1024)\n\npart of #863 \n\nBecause some OTAP fields are optional, in a stream of record batches we\nmay receive subsequent batches with different schemas. Parquet doesn't\nsupport having row groups with different sets of column chunks, which\nmeans we need to know the schema a-priori when the writer is created.\n\nThis PR adds code to normalize the schema of the record batch before\nwriting by:\n- putting all the fields in the same order\n- creating all null/default value columns for any missing column\n\nThe missing columns should have a small overhead when written to disk,\nbecause parquet will either write an entirely empty column chunk for the\nnull column (all null count, no data), or and for all default-value\ncolumns, parquet will use dictionary and RLE encoding by default,\nleading to a small column chunk with a single value value in dict & a\nsingle run for the key.\n\nWhat's unfortunate is that we still materialize an all-null column\nbefore writing with the length of the record batch. This can be\noptimized when run-end encoded arrays are supported in parquet, because\nwe could just create a run array with a single run of null/default\nvalue. The arrow community is currently working on adding support (see\nhttps://github.com/apache/arrow-rs/pull/7713 &\nhttps://github.com/apache/arrow-rs/pull/8069).\n\n---------\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2025-08-28T21:16:23Z",
          "tree_id": "6d21e3690dd86e9b944580e97297add688252dc8",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/fc324c575adcd90158964e2de0725c4a40e32cf2"
        },
        "date": 1756416282481,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 739000,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 22170000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 22170000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 5.6,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 6.66,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 129.67,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 149.18,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 736000,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 22080000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 22080000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-avg",
            "value": 5.82,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 6.87,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 158.69,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 182.05,
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
          "id": "76307ed5b3a61535f642910a778c7fb380192816",
          "message": "[otap-dataflow] Add metrics to fake signal receiver (#1039)\n\nThis PR adds counters for otlp items generated to the fake signal\nexporter, allowing like-for-like comparison of signals sent/received\nbetween it and perf-exporter.\n\n```\n    {\n      \"name\": \"fake_data_generator.receiver.metrics\",\n      \"brief\": \"\",\n      \"attributes\": {\n...\n      },\n      \"metrics\": [\n        {\n          \"name\": \"logs\",\n          \"unit\": \"{log}\",\n          \"brief\": \"Number of logs generated.\",\n          \"instrument\": \"counter\",\n          \"value\": 1000\n        },\n        {\n          \"name\": \"spans\",\n          \"unit\": \"{span}\",\n          \"brief\": \"Number of spans generated.\",\n          \"instrument\": \"counter\",\n          \"value\": 0\n        },\n        {\n          \"name\": \"metrics\",\n          \"unit\": \"{metric}\",\n          \"brief\": \"Number of metrics generated.\",\n          \"instrument\": \"counter\",\n          \"value\": 0\n        }\n      ]\n    }\n```",
          "timestamp": "2025-08-29T22:03:44Z",
          "tree_id": "10c46da34722317a4ba543d896edbdd0bd9c0af8",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/76307ed5b3a61535f642910a778c7fb380192816"
        },
        "date": 1756505540300,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 740333.3333333334,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 22210000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 22210000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 5.76,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 6.67,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 126.84,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 148.59,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 724166.6666666666,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 21725000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 21725000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-avg",
            "value": 5.58,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 6.55,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 172.3,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 211.2,
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
          "id": "9027c6fb3ae37c3e7bba34d3951640dde38b2c2a",
          "message": "[otel-arrow-rust] fix a few bugs splitting traces/logs when creating resized OTAP batches (#1038)\n\npart of #1027 \n\n- fix out of bounds slice access when splitting child if child does not\ncontain parent IDs in the final split of the parent batch\n- stop creating empty record batches for child splits if the child has\nno parent IDs in the range of the parent split\n- `take` child batch from the input arrays so we don't fail the\nassertion in `generic_split` that there's nothing left in the input\n`batches`\n\n---------\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2025-08-29T22:16:14Z",
          "tree_id": "144e0cd9e5a6bd18e0bde624910bbce162cbfe5b",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/9027c6fb3ae37c3e7bba34d3951640dde38b2c2a"
        },
        "date": 1756506296828,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 735500,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 22065000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 22065000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 5.63,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 6.68,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 122.99,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 146.13,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 730833.3333333334,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 21925000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 21925000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-avg",
            "value": 5.62,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 6.6,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 157.46,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 188.27,
            "unit": "MiB"
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
          "id": "52d8145e1eea051ad4493570feaf1b933a50ab7b",
          "message": "Add an endpoint to stop pipelines without stopping the engine to make benchmarking easier (#1040)\n\nThis PR adds an endpoint to stop all currently running pipelines without\nshutting down the service. In the future, we will provide an API to\nstart, stop, update, list, ... the pipelines managed by the engine.\n\nThis PR is useful for the benchmarking infrastructure.\n\nTo stop all the pipeline groups:\n\n```\nPOST http://localhost:8080/pipeline-groups/shutdown\nAccept: application/json\n```",
          "timestamp": "2025-08-29T23:19:33Z",
          "tree_id": "8ea19950e3c4a51d97aa9cd920732397fcded4d3",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/52d8145e1eea051ad4493570feaf1b933a50ab7b"
        },
        "date": 1756510094778,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 736500,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 22095000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 22095000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 5.6,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 6.51,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 130.57,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 150.12,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 735333.3333333334,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 22060000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 22060000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-avg",
            "value": 5.59,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 6.54,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 159.25,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 183.01,
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
          "id": "a515e3d59a12481e8cf3647368bcf1bf7dad9863",
          "message": "[otap-df-enfine] Avoid allocation in info method (#1029)\n\n## Changes\n- Avoid the string allocation by `format!` by directly writing the\nmessage and new line character to the buffer",
          "timestamp": "2025-08-29T23:43:41Z",
          "tree_id": "3c174c82d338648feb7d9c6a7e3347dc5d13654d",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/a515e3d59a12481e8cf3647368bcf1bf7dad9863"
        },
        "date": 1756511539807,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 741166.6666666666,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 22235000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 22235000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 5.62,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 6.67,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 122.24,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 139.15,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 737166.6666666666,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 22115000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 22115000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-avg",
            "value": 5.89,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 7.01,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 166.06,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 198.48,
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
          "id": "32e53e156367b6bea78309ea7d78fbb44c0eaf53",
          "message": "[query-engine] KQL matches regex & more constant folding (#1031)\n\n## Changes\n\n* Adds support for KQL `matches regex` expression\n* Builds on top of #1007 such that all computed values are folded into\nstatics when `try_resolve_static` is invoked",
          "timestamp": "2025-09-02T16:36:40Z",
          "tree_id": "1209d6c4dbb813c725f8da7db470cef942697745",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/32e53e156367b6bea78309ea7d78fbb44c0eaf53"
        },
        "date": 1756831521722,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 741666.6666666666,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 22250000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 22250000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 5.69,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 6.67,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 136.45,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 156.8,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 740833.3333333334,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 22225000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 22225000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-avg",
            "value": 5.75,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 6.85,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 160.27,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 182.85,
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
          "id": "79bb85e29d33536e53b89e3b43a9844ff0553813",
          "message": "[query-engine] Remove ImmutableValueExpression (#1046)\n\n## Changes\n\n* Removes `ImmutableValueExpression` in favor of just using\n`ScalarExpression` directly\n\n## Details\n\nI thought there might be other source of data but the way things worked\nout `ImmutableValueExpression` is really just some extra ceremony\nwrapping `ScalarExpression`.",
          "timestamp": "2025-09-02T17:22:36Z",
          "tree_id": "ce650f1f09f544c625e5ef65426fad78161da566",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/79bb85e29d33536e53b89e3b43a9844ff0553813"
        },
        "date": 1756834263347,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 730000,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 21900000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 21900000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 5.58,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 6.68,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 129.46,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 151.55,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 741500,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 22245000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 22245000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-avg",
            "value": 5.71,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 7.12,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 166.33,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 198.04,
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
          "id": "34640efcbe02e041bc71f0fd5605b93183fbec68",
          "message": "fix building pipelines for proccesors (#1051)\n\ncloses: #1047 \n\n- fix setting the receiver on local processors if upstream component is\nshared\n- fixes attributes processors dropping the out ports from the configured\n`NodeUserConfig`",
          "timestamp": "2025-09-02T20:12:38Z",
          "tree_id": "b4cd1bb5048a6f18f44960323f49d4870e46e68f",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/34640efcbe02e041bc71f0fd5605b93183fbec68"
        },
        "date": 1756844458294,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 732833.3333333334,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 21985000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 21985000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 5.55,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 6.55,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 130.35,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 151.25,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 733833.3333333334,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 22015000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 22015000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-avg",
            "value": 5.7,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 6.78,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 155.42,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 185.16,
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
          "id": "7488f322d29b4a95b28b5a39a3568af8b0d195d3",
          "message": "fix handling of invalid trace and span IDs when encoding OTAP logs (#1052)\n\nFixes issue encoding OTAP when there are trace_ids and span_ids that are\ninvalid length (e.g. from producers who send an empty slice to represent\n\"null\" for these fields).",
          "timestamp": "2025-09-02T21:29:50Z",
          "tree_id": "9202625ec783f3444efa8e5d717b188a49d7bb2a",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/7488f322d29b4a95b28b5a39a3568af8b0d195d3"
        },
        "date": 1756849120476,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 740666.6666666666,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 22220000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 22220000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 5.71,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 6.65,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 123.04,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 143.49,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 736000,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 22080000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 22080000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-avg",
            "value": 5.66,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 6.85,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 167.66,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 200.05,
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
          "id": "9521e861f45552389a2adf99f93817bb253ecff1",
          "message": "[query-engine] Constant folding tweaks (#1049)\n\n## Changes\n\n* Moves folded/copied constant under `StaticScalarExpression`\n\n## Details\n\nWorking towards automatically replacing expressions which are statically\nknown with constant values. The way constants were defined previously we\ncould have static values under `ScalarExpression:Static` or under\n`ScalarExpression::Constant(ConstantScalarExpression::Copy)`. For that\nto work resolution logic needs to handle\\understand the special constant\nlocation everywhere which was proving to be very complex to build and\nlikely difficult to maintain. To avoid all that complexity completely\nwhat this PR does is move folded/copied constants under the static tree\nso they are handled as any other static value.",
          "timestamp": "2025-09-02T22:53:27Z",
          "tree_id": "a402f7e16f47a78176781b867e12382665e2445b",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/9521e861f45552389a2adf99f93817bb253ecff1"
        },
        "date": 1756854404164,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 735500,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 22065000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 22065000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 5.62,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 6.74,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 127.9,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 148.21,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 736500,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 22095000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 22095000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-avg",
            "value": 5.82,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 6.89,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 166.24,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 201.23,
            "unit": "MiB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "129437996+c1ly@users.noreply.github.com",
            "name": "c1ly",
            "username": "c1ly"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "83beec752f40458c4d2f722d19294b1267e85259",
          "message": "[otap-dataflow] add debug processor and noop exporter  (#1012)\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2025-09-02T23:31:03Z",
          "tree_id": "cf8726a06f7c38be4802933dd801a6acd13bdb1c",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/83beec752f40458c4d2f722d19294b1267e85259"
        },
        "date": 1756856486181,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 736833.3333333334,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 22105000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 22105000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 5.66,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 6.61,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 128.97,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 155.84,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 735333.3333333334,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 22060000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 22060000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-avg",
            "value": 5.73,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 6.92,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 164.31,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 193.83,
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
          "id": "dcdbdef991111ab4128e5739a5d4e10ecefb65a2",
          "message": "Small parquet exporter fixes and add example config (#1044)\n\nFixes a few small issues with parquet exporter:\n- fix `flush_when_older_than` missing value not being deserialized as\n`None` by serde\n- fix bug generating unique IDs for ID columns with null values\n\nAlso adds a configuration example for generating parquet data.\n\n---------\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2025-09-03T01:02:37Z",
          "tree_id": "704f61dd972765526050c862c1d6255687037680",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/dcdbdef991111ab4128e5739a5d4e10ecefb65a2"
        },
        "date": 1756862142230,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 740833.3333333334,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 22225000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 22225000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 5.71,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 6.66,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 125.84,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 155.12,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 738833.3333333334,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 22165000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 22165000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-avg",
            "value": 5.63,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 6.57,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 166.87,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 192.35,
            "unit": "MiB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "masslessparticle@gmail.com",
            "name": "Travis Patterson",
            "username": "MasslessParticle"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "d0a6aa096b0794d9e57e72e7349c41ae773fb1b2",
          "message": "fix: remove orphan record.Schema() call in NewLogsMessage (#1053)\n\nI'm playing with otel arrow and noticed this orphaned `record.Schema()`\ncall in NewLogsMessage. This PR removes the call.",
          "timestamp": "2025-09-03T11:13:08Z",
          "tree_id": "6a33ae0015a6ebb430e4c35cd63a91d5812845da",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/d0a6aa096b0794d9e57e72e7349c41ae773fb1b2"
        },
        "date": 1756898507857,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 718833.3333333334,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 21565000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 21565000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 5.63,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 6.72,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 130.57,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 150.35,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 736333.3333333334,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 22090000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 22090000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-avg",
            "value": 5.61,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 6.61,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 157.64,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 186.78,
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
          "id": "8a15a029d24f4ed1a6b2c3a5abd5784dca7ef3dd",
          "message": "Use Arrow row format when sorting for batching if possible (#1043)\n\nCloses: #1026 \n\nImprove batching performance by using the arrow row format for\nmulticolumn sort where possible.\n\nWe do this by creating a reusable function called\n`otap::transform::sort_to_indices` and using this both in the batching\ncode and in the transport optimization encoding code (which is where\nthis was refactored from).",
          "timestamp": "2025-09-03T15:08:28Z",
          "tree_id": "1ae957a13649315aa7a2089eb5172183264427fd",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/8a15a029d24f4ed1a6b2c3a5abd5784dca7ef3dd"
        },
        "date": 1756912600429,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 739000,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 22170000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 22170000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 5.7,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 6.83,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 122.36,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 143.15,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 735833.3333333334,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 22075000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 22075000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-avg",
            "value": 5.75,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 6.98,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 161.47,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 194.4,
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
          "id": "c0daed0738ef6d2e3fd6bacc91d0d4af65b9563f",
          "message": "[PerfTest] - add new hooks for df_engine as a loadgen/backend (#1050)\n\nThis PR adds new hook strategies to the framework in support of using\nthe dataflow engine as a load-generator and backend receiver, and other\ntest suite development QOL fixes:\n\n| Type | Plugin Name | Module | Class | Config Class | Description |\n|------|-------------|--------|-------|--------------|-------------|\n| `send_http_request` | `send_http_request` |\n`lib.impl.strategies.hooks.send_http_request` | `SendHttpRequestHook` |\n`SendHttpRequestConfig` | Hook strategy that sends an HTTP request to a\nconfigured endpoint |\n| `ready_check_http` | `ready_check_http` |\n`lib.impl.strategies.hooks.ready_check_http` | `ReadyCheckHttpHook` |\n`ReadyCheckHttpConfig` | Hook strategy that performs a readiness check\nagainst an HTTP(S) endpoint |\n| `render_template` | `render_template` |\n`lib.impl.strategies.hooks.render_template` | `RenderTemplateHook` |\n`RenderTemplateConfig` | Hook strategy that renders a Jinja2 template\nusing provided variables |\n| `ensure_process` | `ensure_process` |\n`lib.impl.strategies.hooks.process.ensure_process` | `EnsureProcess` |\n`EnsureProcessConfig` | Hook strategy to ensure component specified is\nrunning and hasn't crashed after start |\n| `no_op` | `no_op` | `lib.impl.actions.no_op_action` | `NoOpAction` |\n`NoOpActionConfig` | Step action that does nothing when execute is\ncalled |\n\n---------\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2025-09-03T15:13:27Z",
          "tree_id": "afc091179b04d6ab48bb83ef52a2695de8b81cbf",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/c0daed0738ef6d2e3fd6bacc91d0d4af65b9563f"
        },
        "date": 1756913239917,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 738333.3333333334,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 22150000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 22150000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 5.57,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 6.57,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 133.62,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 153.34,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 740166.6666666666,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 22205000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 22205000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-avg",
            "value": 5.73,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 6.7,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 165.08,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 194.61,
            "unit": "MiB"
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
          "distinct": false,
          "id": "d25cb0dbcf66c1633451d848c97328c6b7ca02a0",
          "message": "Parameter --core-id-range (#1054)\n\nTo isolate the different workloads in our benchmarking infrastructure,\nwe need to be able to allocate subsets of CPU cores to different\ncomponents of the infrastructure (traffic generator, system under test,\nperf collector).",
          "timestamp": "2025-09-03T15:36:56Z",
          "tree_id": "2489eac2ff5e8612e1d6158208bf9d51e5ddd46c",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/d25cb0dbcf66c1633451d848c97328c6b7ca02a0"
        },
        "date": 1756914633295,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 730833.3333333334,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 21925000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 21925000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 5.54,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 6.5,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 124.42,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 143.57,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 735000,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 22050000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 22050000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-avg",
            "value": 5.67,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 6.71,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 171.8,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 199.75,
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
          "id": "f8068a7ef16e0eb44f36480bc92eb72c3ec1e46f",
          "message": "[query-engine] Enable automatic constant folding for pipeline during optimization (#1056)\n\nnt",
          "timestamp": "2025-09-03T17:57:13Z",
          "tree_id": "67d6af40e35cfe0ca8ada24c417bbd06b933d2c2",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/f8068a7ef16e0eb44f36480bc92eb72c3ec1e46f"
        },
        "date": 1756922756197,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 739500,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 22185000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 22185000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 5.58,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 6.71,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 122.48,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 146.72,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 736333.3333333334,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 22090000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 22090000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-avg",
            "value": 5.68,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 6.67,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 163.43,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 192.61,
            "unit": "MiB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "129437996+c1ly@users.noreply.github.com",
            "name": "c1ly",
            "username": "c1ly"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "dda2744fbc51455397ddb786b0cbb5ea4d4b1d57",
          "message": "[otap-dataflow] Register the syslog ref receiver (#1057)\n\nMoved the syslog receiver inside the otap-df-otap crate and registered\nit to the receiver factory\n\n---------\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2025-09-03T19:23:01Z",
          "tree_id": "35cf4612b9b2fd679fda35649cd0484477286f99",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/dda2744fbc51455397ddb786b0cbb5ea4d4b1d57"
        },
        "date": 1756927905331,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 735000,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 22050000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 22050000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 5.55,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 6.47,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 126.04,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 143.4,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 735833.3333333334,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 22075000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 22075000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-avg",
            "value": 5.69,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 6.72,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 169.74,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 196.19,
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
          "id": "8b0135b635bfa496c2e0c1b085ed9cdffa1b6c12",
          "message": "[query-engine] Support accessors into constant values (#1059)\n\n## Changes\n\n* Support accessors into constant values\n\n## Details\n\nThis enables a bunch of things but the main one IMO is efficient value\nlookup:\n\n```\nlet lookup =  parse_json('[\\\"Failure\\\", \\\"Success\\\"]');\nsource | extend Status = coalesce(lookup[NumericStatus], 'Unknown')\n```",
          "timestamp": "2025-09-03T22:24:41Z",
          "tree_id": "e7e4847fd6d84cb9706c4a356c8c32ae7b41c56c",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/8b0135b635bfa496c2e0c1b085ed9cdffa1b6c12"
        },
        "date": 1756938786646,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 739166.6666666666,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 22175000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 22175000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 5.6,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 6.64,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 124.18,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 142.8,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 737833.3333333334,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 22135000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 22135000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-avg",
            "value": 5.62,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 6.62,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 163.22,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 190.3,
            "unit": "MiB"
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
          "id": "b8cf67756b110fbbf33c930b35c0bbc1336feadf",
          "message": "Endpoints to expose the current overall state of the pipelines and to stop pipelines (#1033)\n\nThis PR provides additional visibility into the current state of the\nsystem and pipelines. It is now also possible to stop pipelines through\na shutdown endpoint.\n\nList of admin endpoints added in this PR:\n- Get the status of the specified pipeline: GET\n/pipeline-groups/{pipeline_group_id}/pipelines/{pipeline_id}/status\n- Get the status of all the pipeline groups: GET /pipeline-groups/status\n- Shutdown all pipeline groups: POST /pipeline-groups/shutdown\n\nHeartbeat events are not yet implemented and will be added in a future\nPR.\n\nBackground context: This PR is the first step of the implementation of\nthe reconciler concept.\nA global observed state is now kept up to date by monitoring the changes\nmade to/by the pipelines (and groups).\nIn future PRs, a desired state will be submitted to a reconciler in\norder to update the current configuration/execution of the pipelines.\nThe ultimate goal is to support live reconfigurations and provide full\nobservability of the pipelines' state.",
          "timestamp": "2025-09-04T15:58:48Z",
          "tree_id": "12331ac70781e156ef23de22c44664bdede4a7f9",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/b8cf67756b110fbbf33c930b35c0bbc1336feadf"
        },
        "date": 1757002393431,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 734500,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 22035000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 22035000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 5.64,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 6.64,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 123.85,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 155.14,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 739666.6666666666,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 22190000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 22190000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-avg",
            "value": 5.63,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 6.58,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 156.09,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 181.84,
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
          "id": "445757ca84d962a1a715ca370fa2ee1e572b160d",
          "message": "Avoid flattening dictionaries when unifying record batches during batching (#1042)\n\nCloses: #1025 \n\nWhen concatenating OTAP batches, there are two issues that are a bit\ndifficult WRT to dictionary columns:\n- the combined record batch might overflow the dictionary due to more\nkeys having more cardinality than the key type can support\n- received record batches may have different data types for the same\ndictionary column (it depends on the size of the batch, and the\ncardinality of the column when it was constructed).\n\nBefore this, we were using a simler logic of flattening all\ndictionaries, however in doing this we lose the benefits of dictionary\ncompression.\n\nWith this PR, we no longer flatten the dictionaries. Instead, we attempt\nto determine the correct key size to use and the cast the column for all\nrecord batches to be the correct data type. We try to make the decision\nabout which key type to use in the most efficient way by considering the\ntotal batch size, the possible cardinality of the columns and the\nminimum dictionary key size.\n\n---------\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2025-09-04T16:14:16Z",
          "tree_id": "2d4f8a7c7d5ea6859c3e58a4bcb9733b2c666949",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/445757ca84d962a1a715ca370fa2ee1e572b160d"
        },
        "date": 1757004517443,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 735333.3333333334,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 22060000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 22060000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 5.73,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 6.64,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 126.56,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 152.34,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 737500,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 22125000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 22125000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-avg",
            "value": 5.79,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 6.92,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 166.1,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 188.9,
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
          "id": "113fa57fec200c3fead250ad2ab54b30a790ca8e",
          "message": "[WIP] fix IPC consumer losing state (#1062)\n\nFixes: #1061  (See issue for description of the problem)\n\nThe solution chosen is to remove the arrow bytes representation from the\nOTAP Pdata, and always decode the IPC serialized data in the OTAP\nReceiver.\n\n---------\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2025-09-04T18:33:28Z",
          "tree_id": "051099f2e71538c897e7c464284344aa79b170f6",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/113fa57fec200c3fead250ad2ab54b30a790ca8e"
        },
        "date": 1757011328569,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 731833.3333333334,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 21955000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 21955000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 5.62,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 6.76,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 125.46,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 147.47,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 734666.6666666666,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 22040000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 22040000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-avg",
            "value": 5.69,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 6.66,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 163.8,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 192.38,
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
          "id": "a7a7c3973dfcbc3933c530e3604d6d9b00e0ba07",
          "message": "[otap-dataflow] otap pdata integration for otap_batch_processor (#1015)\n\nIntegration work for otap_batch_processor and otappdata batching\ncapabilities\n\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2025-09-04T21:48:20Z",
          "tree_id": "dc4a02e288e587cc330f7aa384eb55e73d5588e8",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/a7a7c3973dfcbc3933c530e3604d6d9b00e0ba07"
        },
        "date": 1757023018386,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 735833.3333333334,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 22075000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 22075000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 5.65,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 6.72,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 124.11,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 147.47,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 740500,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 22215000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 22215000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-avg",
            "value": 5.69,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 6.71,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 164.29,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 199.43,
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
          "id": "9dbc501ac196c8c5e1396ed27319611d55718875",
          "message": "[query-engine] Constant folding improvements (#1063)\n\n## Changes\n\n* Refactor scalar static resolution methods to use\n`ScalarStaticResolutionResult` alias to improve readability\n* Improve `List`, `Case`, `Conditional`, and `Coalesce` expression\nresolution logic to inspect all inner expressions and fold as much as\npossible during resolution\n* Add tests for `Case` static resolution",
          "timestamp": "2025-09-04T21:53:46Z",
          "tree_id": "bf2702958be80df1061c8ae2a5de86037c6df19c",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/9dbc501ac196c8c5e1396ed27319611d55718875"
        },
        "date": 1757023337487,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 738500,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 22155000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 22155000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 5.67,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 6.63,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 131.45,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 155.44,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 738166.6666666666,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 22145000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 22145000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-avg",
            "value": 5.73,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 6.9,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 162.42,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 190.09,
            "unit": "MiB"
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
          "id": "3278319d262c509be28769c58e6d21d23f45cefa",
          "message": "Improve pipeline draining (#1058)\n\nIn this PR, we no longer send a shutdown control message to all nodes in\na pipeline but only to the receivers. This forces the pipelines to stop\nin the receiver-to-exporter direction. It's not the end of the story for\nachieving graceful draining of the pipelines, but it's a good start.",
          "timestamp": "2025-09-04T23:53:58Z",
          "tree_id": "68b801dc1a8003c1928e494d79496136e27d7d28",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/3278319d262c509be28769c58e6d21d23f45cefa"
        },
        "date": 1757030550324,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 733833.3333333334,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 22015000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 22015000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 5.55,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 6.49,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 127.88,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 151.35,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 733333.3333333334,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 22000000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 22000000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-avg",
            "value": 5.65,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 6.63,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 168.98,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 200.2,
            "unit": "MiB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "129437996+c1ly@users.noreply.github.com",
            "name": "c1ly",
            "username": "c1ly"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "7b9140f27af655d4f12bdcf27d4d0d14c9ff74ce",
          "message": "[otap-dataflow] syslog-cef-receiver update from_config method (#1065)\n\nUpdate the from_config method to use the config parameter\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2025-09-05T00:01:24Z",
          "tree_id": "9bcfeb39de08c455e1fe58081b17e07cb136e6b9",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/7b9140f27af655d4f12bdcf27d4d0d14c9ff74ce"
        },
        "date": 1757031012049,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 733666.6666666666,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 22010000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 22010000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 5.56,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 6.48,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 123.86,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 146.34,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 735833.3333333334,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 22075000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 22075000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-avg",
            "value": 5.68,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 6.77,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 167.22,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 199.42,
            "unit": "MiB"
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
          "distinct": false,
          "id": "13c918b7975d42c5690e344a459a4ed320fadac3",
          "message": "Cleanup of the OTLP crate (#1069)\n\nMost of the code in the `otlp` crate was no longer used, as it had been\nmigrated to the OTAP pipeline.\nThe code still in use (shared between OTLP and OTAP) was moved to the\nOTAP crate.\nRationalization of `Cargo.toml` to use only one version of `tonic` and\n`prost`.\nUpdated `README.md`.\n\nFuture PR: The next step is to migrate to `tonic/prost` 0.14.",
          "timestamp": "2025-09-05T16:36:29Z",
          "tree_id": "48a66120ea4d8acc9586103dc0bf9e2013c2d3e6",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/13c918b7975d42c5690e344a459a4ed320fadac3"
        },
        "date": 1757090691799,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 734333.3333333334,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 22030000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 22030000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 5.63,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 6.61,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 128.12,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 148.34,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 741333.3333333334,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 22240000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 22240000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-avg",
            "value": 5.8,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 7.13,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 171.61,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 199.42,
            "unit": "MiB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "m.salib@f5.com",
            "name": "msalib",
            "username": "msalib"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "1d8c77bd7a46184722766aff9aba126d5d12f3a6",
          "message": "fix: metrics splitter in the batching subsystem (#1071)\n\nCloses #1027.",
          "timestamp": "2025-09-05T19:04:38Z",
          "tree_id": "a6da9a690be94babb8863ac9af291396a039d036",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/1d8c77bd7a46184722766aff9aba126d5d12f3a6"
        },
        "date": 1757099543618,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 735833.3333333334,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 22075000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 22075000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 5.6,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 6.62,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 133.68,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 155.4,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 740833.3333333334,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 22225000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 22225000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-avg",
            "value": 5.78,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 6.71,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 168,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 199.89,
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
          "id": "c29252384faee716b6d1344af9b7c482579d33b6",
          "message": "fix issue decoding attribute key-values with special characters (#1077)\n\nFixes: #1060 \nFixes: #1070 \n\nWhen we decoded an attribute, we weren't advancing the `RawKeyValue`'s\nposition in the buffer to the start of the next field, we'd just advance\nit to the byte after the field's tag (e.g. where the len starts). Then\nwhen we go to read the values, sometimes we'd parse each byte in the key\nas the start of a new proto field as if it was the start of a new field\nby treating it like a variant and advancing. For some key values, we'd\njust happen to stumble across the start of the value and everything\nwould be fine. But depending on the bytes, sometimes we'd miss it and\ndecode the attribute as empty.\n\n---------\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2025-09-05T19:28:10Z",
          "tree_id": "924d5875e844871e1c7be472c3c0e10ee38b68bb",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/c29252384faee716b6d1344af9b7c482579d33b6"
        },
        "date": 1757100979695,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 730000,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 21900000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 21900000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 5.52,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 6.41,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 124.27,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 151.07,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 738833.3333333334,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 22165000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 22165000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-avg",
            "value": 5.74,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 6.81,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 163.82,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 189.53,
            "unit": "MiB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "129437996+c1ly@users.noreply.github.com",
            "name": "c1ly",
            "username": "c1ly"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "5da284414e9b14f678344b51e5292229e4b5f8d2",
          "message": "update tonic version to 0.14 (#1079)\n\n- update tonic to 0.14\n- added the tonic-prost crate that the prost feature was migrated into\n- Updated the tonic-build crate to tonic-prost-build\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2025-09-05T21:53:42Z",
          "tree_id": "360300cf7b208fd8955b3aaf3ac5b83247bae8b6",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/5da284414e9b14f678344b51e5292229e4b5f8d2"
        },
        "date": 1757109718381,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 730666.6666666666,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 21920000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 21920000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 5.59,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 6.8,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 124.81,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 145.65,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 735333.3333333334,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 22060000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 22060000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-avg",
            "value": 5.73,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 6.75,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 162.15,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 189.66,
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
          "id": "18996ac489c7ed98f315b803275a3dc112844fc5",
          "message": "[query-engine] Regex capture support (#1072)\n\n## Changes\n\n* Adds expression and recordset engine implementation for simple regex\ncapture (single group, first match)\n* Adds KQL parsing for `extract` using the new expression",
          "timestamp": "2025-09-05T23:12:53Z",
          "tree_id": "933988c5e05a1b54861e5eb1d9d4a94024ab517a",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/18996ac489c7ed98f315b803275a3dc112844fc5"
        },
        "date": 1757117816017,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 734000,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 22020000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 22020000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 5.69,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 6.73,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 124.42,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 149.38,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 735000,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 22050000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 22050000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-avg",
            "value": 5.68,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 6.9,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 159.12,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 185.11,
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
          "id": "07fcadbe2fd2208f4f69e0bcf59abd649c66ee23",
          "message": "chore(deps): update github workflow dependencies (#1086)\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n|---|---|---|---|\n|\n[benchmark-action/github-action-benchmark](https://redirect.github.com/benchmark-action/github-action-benchmark)\n| action | patch | `v1.20.4` -> `v1.20.7` |\n|\n[codecov/codecov-action](https://redirect.github.com/codecov/codecov-action)\n| action | patch | `v5.5.0` -> `v5.5.1` |\n|\n[github/codeql-action](https://redirect.github.com/github/codeql-action)\n| action | minor | `v3.29.11` -> `v3.30.1` |\n| [go](https://redirect.github.com/actions/go-versions) | uses-with |\npatch | `1.25.0` -> `1.25.1` |\n|\n[taiki-e/install-action](https://redirect.github.com/taiki-e/install-action)\n| action | minor | `v2.58.21` -> `v2.59.1` |\n\n---\n\n### Release Notes\n\n<details>\n<summary>benchmark-action/github-action-benchmark\n(benchmark-action/github-action-benchmark)</summary>\n\n###\n[`v1.20.7`](https://redirect.github.com/benchmark-action/github-action-benchmark/blob/HEAD/CHANGELOG.md#v1207---06-Sep-2025)\n\n[Compare\nSource](https://redirect.github.com/benchmark-action/github-action-benchmark/compare/v1.20.5...v1.20.7)\n\n- **fix** improve parsing for custom benchmarks\n([#&#8203;323](https://redirect.github.com/benchmark-action/github-action-benchmark/issues/323))\n\n###\n[`v1.20.5`](https://redirect.github.com/benchmark-action/github-action-benchmark/blob/HEAD/CHANGELOG.md#v1205---02-Sep-2025)\n\n[Compare\nSource](https://redirect.github.com/benchmark-action/github-action-benchmark/compare/v1.20.4...v1.20.5)\n\n- **feat** allow to parse generic cargo bench/criterion units\n([#&#8203;280](https://redirect.github.com/benchmark-action/github-action-benchmark/issues/280))\n- **fix** add summary even when failure threshold is surpassed\n([#&#8203;285](https://redirect.github.com/benchmark-action/github-action-benchmark/issues/285))\n- **fix** time units are not normalized\n([#&#8203;318](https://redirect.github.com/benchmark-action/github-action-benchmark/issues/318))\n\n</details>\n\n<details>\n<summary>codecov/codecov-action (codecov/codecov-action)</summary>\n\n###\n[`v5.5.1`](https://redirect.github.com/codecov/codecov-action/blob/HEAD/CHANGELOG.md#v551)\n\n[Compare\nSource](https://redirect.github.com/codecov/codecov-action/compare/v5.5.0...v5.5.1)\n\n##### What's Changed\n\n- fix: overwrite pr number on fork by\n[@&#8203;thomasrockhu-codecov](https://redirect.github.com/thomasrockhu-codecov)\nin\n[#&#8203;1871](https://redirect.github.com/codecov/codecov-action/pull/1871)\n- build(deps): bump actions/checkout from 4.2.2 to 5.0.0 by\n[@&#8203;app/dependabot](https://redirect.github.com/app/dependabot) in\n[#&#8203;1868](https://redirect.github.com/codecov/codecov-action/pull/1868)\n- build(deps): bump github/codeql-action from 3.29.9 to 3.29.11 by\n[@&#8203;app/dependabot](https://redirect.github.com/app/dependabot) in\n[#&#8203;1867](https://redirect.github.com/codecov/codecov-action/pull/1867)\n- fix: update to use local app/ dir by\n[@&#8203;thomasrockhu-codecov](https://redirect.github.com/thomasrockhu-codecov)\nin\n[#&#8203;1872](https://redirect.github.com/codecov/codecov-action/pull/1872)\n- docs: fix typo in README by\n[@&#8203;datalater](https://redirect.github.com/datalater) in\n[#&#8203;1866](https://redirect.github.com/codecov/codecov-action/pull/1866)\n- Document a `codecov-cli` version reference example by\n[@&#8203;webknjaz](https://redirect.github.com/webknjaz) in\n[#&#8203;1774](https://redirect.github.com/codecov/codecov-action/pull/1774)\n- build(deps): bump github/codeql-action from 3.28.18 to 3.29.9 by\n[@&#8203;app/dependabot](https://redirect.github.com/app/dependabot) in\n[#&#8203;1861](https://redirect.github.com/codecov/codecov-action/pull/1861)\n- build(deps): bump ossf/scorecard-action from 2.4.1 to 2.4.2 by\n[@&#8203;app/dependabot](https://redirect.github.com/app/dependabot) in\n[#&#8203;1833](https://redirect.github.com/codecov/codecov-action/pull/1833)\n\n**Full Changelog**:\n<https://github.com/codecov/codecov-action/compare/v5.5.0..v5.5.1>\n\n</details>\n\n<details>\n<summary>github/codeql-action (github/codeql-action)</summary>\n\n###\n[`v3.30.1`](https://redirect.github.com/github/codeql-action/compare/v3.30.0...v3.30.1)\n\n[Compare\nSource](https://redirect.github.com/github/codeql-action/compare/v3.30.0...v3.30.1)\n\n###\n[`v3.30.0`](https://redirect.github.com/github/codeql-action/releases/tag/v3.30.0)\n\n[Compare\nSource](https://redirect.github.com/github/codeql-action/compare/v3.29.11...v3.30.0)\n\n##### CodeQL Action Changelog\n\nSee the [releases\npage](https://redirect.github.com/github/codeql-action/releases) for the\nrelevant changes to the CodeQL CLI and language packs.\n\n##### 3.30.0 - 01 Sep 2025\n\n- Reduce the size of the CodeQL Action, speeding up workflows by\napproximately 4 seconds.\n[#&#8203;3054](https://redirect.github.com/github/codeql-action/pull/3054)\n\nSee the full\n[CHANGELOG.md](https://redirect.github.com/github/codeql-action/blob/v3.30.0/CHANGELOG.md)\nfor more information.\n\n</details>\n\n<details>\n<summary>actions/go-versions (go)</summary>\n\n###\n[`v1.25.1`](https://redirect.github.com/actions/go-versions/releases/tag/1.25.1-17451174567):\n1.25.1\n\n[Compare\nSource](https://redirect.github.com/actions/go-versions/compare/1.25.0-16925932082...1.25.1-17451174567)\n\nGo 1.25.1\n\n</details>\n\n<details>\n<summary>taiki-e/install-action (taiki-e/install-action)</summary>\n\n###\n[`v2.59.1`](https://redirect.github.com/taiki-e/install-action/blob/HEAD/CHANGELOG.md#100---2021-12-30)\n\n[Compare\nSource](https://redirect.github.com/taiki-e/install-action/compare/v2.59.0...v2.59.1)\n\nInitial release\n\n[Unreleased]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.59.1...HEAD\n\n[2.59.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.59.0...v2.59.1\n\n[2.59.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.33...v2.59.0\n\n[2.58.33]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.32...v2.58.33\n\n[2.58.32]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.31...v2.58.32\n\n[2.58.31]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.30...v2.58.31\n\n[2.58.30]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.29...v2.58.30\n\n[2.58.29]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.28...v2.58.29\n\n[2.58.28]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.27...v2.58.28\n\n[2.58.27]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.26...v2.58.27\n\n[2.58.26]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.25...v2.58.26\n\n[2.58.25]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.24...v2.58.25\n\n[2.58.24]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.23...v2.58.24\n\n[2.58.23]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.22...v2.58.23\n\n[2.58.22]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.21...v2.58.22\n\n[2.58.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.20...v2.58.21\n\n[2.58.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.19...v2.58.20\n\n[2.58.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.18...v2.58.19\n\n[2.58.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.17...v2.58.18\n\n[2.58.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.16...v2.58.17\n\n[2.58.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.15...v2.58.16\n\n[2.58.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.14...v2.58.15\n\n[2.58.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.13...v2.58.14\n\n[2.58.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.12...v2.58.13\n\n[2.58.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.11...v2.58.12\n\n[2.58.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.10...v2.58.11\n\n[2.58.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.9...v2.58.10\n\n[2.58.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.8...v2.58.9\n\n[2.58.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.7...v2.58.8\n\n[2.58.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.6...v2.58.7\n\n[2.58.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.5...v2.58.6\n\n[2.58.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.4...v2.58.5\n\n[2.58.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.3...v2.58.4\n\n[2.58.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.2...v2.58.3\n\n[2.58.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.1...v2.58.2\n\n[2.58.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.0...v2.58.1\n\n[2.58.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.8...v2.58.0\n\n[2.57.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.7...v2.57.8\n\n[2.57.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.6...v2.57.7\n\n[2.57.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.5...v2.57.6\n\n[2.57.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.4...v2.57.5\n\n[2.57.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.3...v2.57.4\n\n[2.57.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.2...v2.57.3\n\n[2.57.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.1...v2.57.2\n\n[2.57.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.0...v2.57.1\n\n[2.57.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.24...v2.57.0\n\n[2.56.24]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.23...v2.56.24\n\n[2.56.23]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.22...v2.56.23\n\n[2.56.22]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.21...v2.56.22\n\n[2.56.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.20...v2.56.21\n\n[2.56.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.19...v2.56.20\n\n[2.56.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.18...v2.56.19\n\n[2.56.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.17...v2.56.18\n\n[2.56.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.16...v2.56.17\n\n[2.56.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.15...v2.56.16\n\n[2.56.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.14...v2.56.15\n\n[2.56.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.13...v2.56.14\n\n[2.56.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.12...v2.56.13\n\n[2.56.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.11...v2.56.12\n\n[2.56.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.10...v2.56.11\n\n[2.56.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.9...v2.56.10\n\n[2.56.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.8...v2.56.9\n\n[2.56.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.7...v2.56.8\n\n[2.56.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.6...v2.56.7\n\n[2.56.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.5...v2.56.6\n\n[2.56.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.4...v2.56.5\n\n[2.56.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.3...v2.56.4\n\n[2.56.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.2...v2.56.3\n\n[2.56.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.1...v2.56.2\n\n[2.56.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.0...v2.56.1\n\n[2.56.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.55.4...v2.56.0\n\n[2.55.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.55.3...v2.55.4\n\n[2.55.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.55.2...v2.55.3\n\n[2.55.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.55.1...v2.55.2\n\n[2.55.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.55.0...v2.55.1\n\n[2.55.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.54.3...v2.55.0\n\n[2.54.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.54.2...v2.54.3\n\n[2.54.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.54.1...v2.54.2\n\n[2.54.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.54.0...v2.54.1\n\n[2.54.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.53.2...v2.54.0\n\n[2.53.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.53.1...v2.53.2\n\n[2.53.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.53.0...v2.53.1\n\n[2.53.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.8...v2.53.0\n\n[2.52.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.7...v2.52.8\n\n[2.52.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.6...v2.52.7\n\n[2.52.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.5...v2.52.6\n\n[2.52.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.4...v2.52.5\n\n[2.52.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.3...v2.52.4\n\n[2.52.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.2...v2.52.3\n\n[2.52.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.1...v2.52.2\n\n[2.52.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.0...v2.52.1\n\n[2.52.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.51.3...v2.52.0\n\n[2.51.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.51.2...v2.51.3\n\n[2.51.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.51.1...v2.51.2\n\n[2.51.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.51.0...v2.51.1\n\n[2.51.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.10...v2.51.0\n\n[2.50.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.9...v2.50.10\n\n[2.50.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.8...v2.50.9\n\n[2.50.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.7...v2.50.8\n\n[2.50.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.6...v2.50.7\n\n[2.50.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.5...v2.50.6\n\n[2.50.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.4...v2.50.5\n\n[2.50.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.3...v2.50.4\n\n[2.50.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.2...v2.50.3\n\n[2.50.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.1...v2.50.2\n\n[2.50.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.0...v2.50.1\n\n[2.50.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.50...v2.50.0\n\n[2.49.50]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.49...v2.49.50\n\n[2.49.49]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.48...v2.49.49\n\n[2.49.48]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.47...v2.49.48\n\n[2.49.47]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.46...v2.49.47\n\n[2.49.46]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.45...v2.49.46\n\n[2.49.45]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.44...v2.49.45\n\n[2.49.44]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.43...v2.49.44\n\n[2.49.43]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.42...v2.49.43\n\n[2.49.42]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.41...v2.49.42\n\n[2.49.41]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.40...v2.49.41\n\n[2.49.40]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.39...v2.49.40\n\n[2.49.39]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.38...v2.49.39\n\n[2.49.38]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.37...v2.49.38\n\n[2.49.37]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.36...v2.49.37\n\n[2.49.36]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.35...v2.49.36\n\n[2.49.35]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.34...v2.49.35\n\n[2.49.34]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.33...v2.49.34\n\n[2.49.33]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.32...v2.49.33\n\n[2.49.32]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.31...v2.49.32\n\n[2.49.31]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.30...v2.49.31\n\n[2.49.30]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.29...v2.49.30\n\n[2.49.29]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.28...v2.49.29\n\n[2.49.28]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.27...v2.49.28\n\n[2.49.27]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.26...v2.49.27\n\n[2.49.26]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.25...v2.49.26\n\n[2.49.25]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.24...v2.49.25\n\n[2.49.24]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.23...v2.49.24\n\n[2.49.23]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.22...v2.49.23\n\n[2.49.22]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.21...v2.49.22\n\n[2.49.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.20...v2.49.21\n\n[2.49.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.19...v2.49.20\n\n[2.49.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.18...v2.49.19\n\n[2.49.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.17...v2.49.18\n\n[2.49.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.16...v2.49.17\n\n[2.49.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.15...v2.49.16\n\n[2.49.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.14...v2.49.15\n\n[2.49.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.13...v2.49.14\n\n[2.49.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.12...v2.49.13\n\n[2.49.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.11...v2.49.12\n\n[2.49.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.10...v2.49.11\n\n[2.49.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.9...v2.49.10\n\n[2.49.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.8...v2.49.9\n\n[2.49.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.7...v2.49.8\n\n[2.49.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.6...v2.49.7\n\n[2.49.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.5...v2.49.6\n\n[2.49.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.4...v2.49.5\n\n[2.49.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.3...v2.49.4\n\n[2.49.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.2...v2.49.3\n\n[2.49.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.1...v2.49.2\n\n[2.49.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.0...v2.49.1\n\n[2.49.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.22...v2.49.0\n\n[2.48.22]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.21...v2.48.22\n\n[2.48.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.20...v2.48.21\n\n[2.48.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.19...v2.48.20\n\n[2.48.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.18...v2.48.19\n\n[2.48.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.17...v2.48.18\n\n[2.48.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.16...v2.48.17\n\n[2.48.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.15...v2.48.16\n\n[2.48.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.14...v2.48.15\n\n[2.48.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.13...v2.48.14\n\n[2.48.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.12...v2.48.13\n\n[2.48.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.11...v2.48.12\n\n[2.48.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.10...v2.48.11\n\n[2.48.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.9...v2.48.10\n\n[2.48.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.8...v2.48.9\n\n[2.48.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.7...v2.48.8\n\n[2.48.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.6...v2.48.7\n\n[2.48.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.5...v2.48.6\n\n[2.48.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.4...v2.48.5\n\n[2.48.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.3...v2.48.4\n\n[2.48.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.2...v2.48.3\n\n[2.48.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.1...v2.48.2\n\n[2.48.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.0...v2.48.1\n\n[2.48.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.32...v2.48.0\n\n[2.47.32]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.31...v2.47.32\n\n[2.47.31]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.30...v2.47.31\n\n[2.47.30]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.29...v2.47.30\n\n[2.47.29]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.28...v2.47.29\n\n[2.47.28]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.27...v2.47.28\n\n[2.47.27]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.26...v2.47.27\n\n[2.47.26]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.25...v2.47.26\n\n[2.47.25]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.24...v2.47.25\n\n[2.47.24]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.23...v2.47.24\n\n[2.47.23]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.22...v2.47.23\n\n[2.47.22]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.21...v2.47.22\n\n[2.47.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.20...v2.47.21\n\n[2.47.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.19...v2.47.20\n\n[2.47.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.18...v2.47.19\n\n[2.47.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.17...v2.47.18\n\n[2.47.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.16...v2.47.17\n\n[2.47.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.15...v2.47.16\n\n[2.47.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.14...v2.47.15\n\n[2.47.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.13...v2.47.14\n\n[2.47.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.12...v2.47.13\n\n[2.47.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.11...v2.47.12\n\n[2.47.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.10...v2.47.11\n\n[2.47.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.9...v2.47.10\n\n[2.47.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.8...v2.47.9\n\n[2.47.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.7...v2.47.8\n\n[2.47.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.6...v2.47.7\n\n[2.47.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.5...v2.47.6\n\n[2.47.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.4...v2.47.5\n\n[2.47.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.3...v2.47.4\n\n[2.47.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.2...v2.47.3\n\n[2.47.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.1...v2.47.2\n\n[2.47.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.0...v2.47.1\n\n[2.47.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.20...v2.47.0\n\n[2.46.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.19...v2.46.20\n\n[2.46.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.18...v2.46.19\n\n[2.46.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.17...v2.46.18\n\n[2.46.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.16...v2.46.17\n\n[2.46.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.15...v2.46.16\n\n[2.46.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.14...v2.46.15\n\n[2.46.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.13...v2.46.14\n\n[2.46.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.12...v2.46.13\n\n[2.46.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.11...v2.46.12\n\n[2.46.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.10...v2.46.11\n\n[2.46.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.9...v2.46.10\n\n[2.46.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.8...v2.46.9\n\n[2.46.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.7...v2.46.8\n\n[2.46.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.6...v2.46.7\n\n[2.46.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.5...v2.46.6\n\n[2.46.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.4...v2.46.5\n\n[2.46.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.3...v2.46.4\n\n[2.46.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.2...v2.46.3\n\n[2.46.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.1...v2.46.2\n\n[2.46.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.0...v2.46.1\n\n[2.46.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.15...v2.46.0\n\n[2.45.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.14...v2.45.15\n\n[2.45.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.13...v2.45.14\n\n[2.45.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.12...v2.45.13\n\n[2.45.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.11...v2.45.12\n\n[2.45.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.10...v2.45.11\n\n[2.45.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.9...v2.45.10\n\n[2.45.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.8...v2.45.9\n\n[2.45.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.7...v2.45.8\n\n[2.45.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.6...v2.45.7\n\n[2.45.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.5...v2.45.6\n\n[2.45.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.4...v2.45.5\n\n[2.45.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.3...v2.45.4\n\n[2.45.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.2...v2.45.3\n\n[2.45.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.1...v2.45.2\n\n[2.45.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.0...v2.45.1\n\n[2.45.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.72...v2.45.0\n\n[2.44.72]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.71...v2.44.72\n\n[2.44.71]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.70...v2.44.71\n\n[2.44.70]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.69...v2.44.70\n\n[2.44.69]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.68...v2.44.69\n\n[2.44.68]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.67...v2.44.68\n\n[2.44.67]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.66...v2.44.67\n\n[2.44.66]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.65...v2.44.66\n\n[2.44.65]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.64...v2.44.65\n\n[2.44.64]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.63...v2.44.64\n\n[2.44.63]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.62...v2.44.63\n\n[2.44.62]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.61...v2.44.62\n\n[2.44.61]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.60...v2.44.61\n\n[2.44.60]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.59...v2.44.60\n\n[2.44.59]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.58...v2.44.59\n\n[2.44.58]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.57...v2.44.58\n\n[2.44.57]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.56...v2.44.57\n\n[2.44.56]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.55...v2.44.56\n\n[2.44.55]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.54...v2.44.55\n\n[2.44.54]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.53...v2.44.54\n\n[2.44.53]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.52...v2.44.53\n\n[2.44.52]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.51...v2.44.52\n\n[2.44.51]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.50...v2.44.51\n\n[2.44.50]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.49...v2.44.50\n\n[2.44.49]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.48...v2.44.49\n\n[2.44.48]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.47...v2.44.48\n\n[2.44.47]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.46...v2.44.47\n\n[2.44.46]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.45...v2.44.46\n\n[2.44.45]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.44...v2.44.45\n\n[2.44.44]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.43...v2.44.44\n\n[2.44.43]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.42...v2.44.43\n\n[2.44.42]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.41...v2.44.42\n\n[2.44.41]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.40...v2.44.41\n\n[2.44.40]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.39...v2.44.40\n\n[2.44.39]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.38...v2.44.39\n\n[2.44.38]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.37...v2.44.38\n\n[2.44.37]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.36...v2.44.37\n\n[2.44.36]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.35...v2.44.36\n\n[2.44.35]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.34...v2.44.35\n\n[2.44.34]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.33...v2.44.34\n\n[2.44.33]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.32...v2.44.33\n\n[2.44.32]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.31...v2.44.32\n\n[2.44.31]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.30...v2.44.31\n\n[2.44.30]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.29...v2.44.30\n\n[2.44.29]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.28...v2.44.29\n\n[2.44.28]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.27...v2.44.28\n\n[2.44.27]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.26...v2.44.27\n\n[2.44.26]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.25...v2.44.26\n\n[2.44.25]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.24...v2.44.25\n\n[2.44.24]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.23...v2.44.24\n\n[2.44.23]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.22...v2.44.23\n\n[2.44.22]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.21...v2.44.22\n\n[2.44.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.20...v2.44.21\n\n[2.44.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.19...v2.44.20\n\n[2.44.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.18...v2.44.19\n\n[2.44.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.17...v2.44.18\n\n[2.44.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.16...v2.44.17\n\n[2.44.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.15...v2.44.16\n\n[2.44.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.14...v2.44.15\n\n[2.44.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.13...v2.44.14\n\n[2.44.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.12...v2.44.13\n\n[2.44.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.11...v2.44.12\n\n[2.44.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.10...v2.44.11\n\n[2.44.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.9...v2.44.10\n\n[2.44.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.8...v2.44.9\n\n[2.44.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.7...v2.44.8\n\n[2.44.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.6...v2.44.7\n\n[2.44.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.5...v2.44.6\n\n[2.44.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.4...v2.44.5\n\n[2.44.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.3...v2.44.4\n\n[2.44.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.2...v2.44.3\n\n[2.44.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.1...v2.44.2\n\n[2.44.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.0...v2.44.1\n\n[2.44.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.43.7...v2.44.0\n\n[2.43.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.43.6...v2.43.7\n\n[2.43.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.43.5...v2.43.6\n\n[2.43.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.43.4...v2.43.5\n\n[2.43.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.43.3...v2.43.4\n\n[2.43.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.43.2...v2.43.3\n\n[2.43.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.43.1...v2.43.2\n\n[2.43.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.43.0...v2.43.1\n\n[2.43.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.42...v2.43.0\n\n[2.42.42]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.41...v2.42.42\n\n[2.42.41]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.40...v2.42.41\n\n[2.42.40]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.39...v2.42.40\n\n[2.42.39]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.38...v2.42.39\n\n[2.42.38]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.37...v2.42.38\n\n[2.42.37]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.36...v2.42.37\n\n[2.42.36]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.35...v2.42.36\n\n[2.42.35]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.34...v2.42.35\n\n[2.42.34]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.33...v2.42.34\n\n[2.42.33]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.32...v2.42.33\n\n[2.42.32]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.31...v2.42.32\n\n[2.42.31]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.30...v2.42.31\n\n[2.42.30]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.29...v2.42.30\n\n[2.42.29]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.28...v2.42.29\n\n[2.42.28]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.27...v2.42.28\n\n[2.42.27]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.26...v2.42.27\n\n[2.42.26]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.25...v2.42.26\n\n[2.42.25]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.24...v2.42.25\n\n[2.42.24]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.23...v2.42.24\n\n[2.42.23]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.22...v2.42.23\n\n[2.42.22]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.21...v2.42.22\n\n[2.42.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.20...v2.42.21\n\n[2.42.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.19...v2.42.20\n\n[2.42.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.18...v2.42.19\n\n[2.42.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.17...v2.42.18\n\n[2.42.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.16...v2.42.17\n\n[2.42.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.15...v2.42.16\n\n[2.42.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.14...v2.42.15\n\n[2.42.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.13...v2.42.14\n\n[2.42.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.12...v2.42.13\n\n[2.42.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.11...v2.42.12\n\n[2.42.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.10...v2.42.11\n\n[2.42.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.9...v2.42.10\n\n[2.42.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.8...v2.42.9\n\n[2.42.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.7...v2.42.8\n\n[2.42.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.6...v2.42.7\n\n[2.42.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.5...v2.42.6\n\n[2.42.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.4...v2.42.5\n\n[2.42.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.3...v2.42.4\n\n[2.42.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.2...v2.42.3\n\n[2.42.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.1...v2.42.2\n\n[2.42.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.0...v2.42.1\n\n[2.42.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.18...v2.42.0\n\n[2.41.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.17...v2.41.18\n\n[2.41.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.16...v2.41.17\n\n[2.41.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.15...v2.41.16\n\n[2.41.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.14...v2.41.15\n\n[2.41.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.13...v2.41.14\n\n[2.41.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.12...v2.41.13\n\n[2.41.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.11...v2.41.12\n\n[2.41.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.10...v2.41.11\n\n[2.41.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.9...v2.41.10\n\n[2.41.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.8...v2.41.9\n\n[2.41.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.7...v2.41.8\n\n[2.41.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.6...v2.41.7\n\n[2.41.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.5...v2.41.6\n\n[2.41.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.4...v2.41.5\n\n[2.41.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.3...v2.41.4\n\n[2.41.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.2...v2.41.3\n\n[2.41.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.1...v2.41.2\n\n[2.41.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.0...v2.41.1\n\n[2.41.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.40.2...v2.41.0\n\n[2.40.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.40.1...v2.40.2\n\n[2.40.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.40.0...v2.40.1\n\n[2.40.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.39.2...v2.40.0\n\n[2.39.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.39.1...v2.39.2\n\n[2.39.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.39.0...v2.39.1\n\n[2.39.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.38.7...v2.39.0\n\n[2.38.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.38.6...v2.38.7\n\n[2.38.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.38.5...v2.38.6\n\n[2.38.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.38.4...v2.38.5\n\n[2.38.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.38.3...v2.38.4\n\n[2.38.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.38.2...v2.38.3\n\n[2.38.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.38.1...v2.38.2\n\n[2.38.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.38.0...v2.38.1\n\n[2.38.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.37.0...v2.38.0\n\n[2.37.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.36.0...v2.37.0\n\n[2.36.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.35.0...v2.36.0\n\n[2.35.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.34.3...v2.35.0\n\n[2.34.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.34.2...v2.34.3\n\n[2.34.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.34.1...v2.34.2\n\n[2.34.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.34.0...v2.34.1\n\n[2.34.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.36...v2.34.0\n\n[2.33.36]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.35...v2.33.36\n\n[2.33.35]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.34...v2.33.35\n\n[2.33.34]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.33...v2.33.34\n\n[2.33.33]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.32...v2.33.33\n\n[2.33.32]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.31...v2.33.32\n\n[2.33.31]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.30...v2.33.31\n\n[2.33.30]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.29...v2.33.30\n\n[2.33.29]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.28...v2.33.29\n\n[2.33.28]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.27...v2.33.28\n\n[2.33.27]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.26...v2.33.27\n\n[2.33.26]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.25...v2.33.26\n\n[2.33.25]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.24...v2.33.25\n\n[2.33.24]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.23...v2.33.24\n\n[2.33.23]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.22...v2.33.23\n\n[2.33.22]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.21...v2.33.22\n\n[2.33.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.20...v2.33.21\n\n[2.33.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.19...v2.33.20\n\n[2.33.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.18...v2.33.19\n\n[2.33.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.17...v2.33.18\n\n[2.33.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.16...v2.33.17\n\n[2.33.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.15...v2.33.16\n\n[2.33.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.14...v2.33.15\n\n[2.33.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.13...v2.33.14\n\n[2.33.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.12...v2.33.13\n\n[2.33.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.11...v2.33.12\n\n[2.33.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.10...v2.33.11\n\n[2.33.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.9...v2.33.10\n\n[2.33.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.8...v2.33.9\n\n[2.33.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.7...v2.33.8\n\n[2.33.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.6...v2.33.7\n\n[2.33.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.5...v2.33.6\n\n[2.33.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.4...v2.33.5\n\n[2.33.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.3...v2.33.4\n\n[2.33.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.2...v2.33.3\n\n[2.33.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.1...v2.33.2\n\n[2.33.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.0...v2.33.1\n\n[2.33.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.20...v2.33.0\n\n[2.32.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.19...v2.32.20\n\n[2.32.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.18...v2.32.19\n\n[2.32.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.17...v2.32.18\n\n[2.32.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.16...v2.32.17\n\n[2.32.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.15...v2.32.16\n\n[2.32.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.14...v2.32.15\n\n[2.32.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.13...v2.32.14\n\n[2.32.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.12...v2.32.13\n\n[2.32.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.11...v2.32.12\n\n[2.32.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.10...v2.32.11\n\n[2.32.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.9...v2.32.10\n\n[2.32.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.8...v2.32.9\n\n[2.32.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.7...v2.32.8\n\n[2.32.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.6...v2.32.7\n\n[2.32.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.5...v2.32.6\n\n[2.32.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.4...v2.32.5\n\n[2.32.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.3...v2.32.4\n\n[2.32.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.2...v2.32.3\n\n[2.32.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.1...v2.32.2\n\n[2.32.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.0...v2.32.1\n\n[2.32.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.31.3...v2.32.0\n\n[2.31.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.31.2...v2.31.3\n\n[2.31.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.31.1...v2.31.2\n\n[2.31.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.31.0...v2.31.1\n\n[2.31.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.30.0...v2.31.0\n\n[2.30.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.29.8...v2.30.0\n\n[2.29.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.29.7...v2.29.8\n\n[2.29.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.29.6...v2.29.7\n\n[2.29.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.29.5...v2.29.6\n\n[2.29.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.29.4...v2.29.5\n\n[2.29.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.29.3...v2.29.4\n\n[2.29.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.29.2...v2.29.3\n\n[2.29.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.29.1...v2.29.2\n\n[2.29.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.29.0...v2.29.1\n\n[2.29.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.16...v2.29.0\n\n[2.28.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.15...v2.28.16\n\n[2.28.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.14...v2.28.15\n\n[2.28.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.13...v2.28.14\n\n[2.28.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.12...v2.28.13\n\n[2.28.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.11...v2.28.12\n\n[2.28.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.10...v2.28.11\n\n[2.28.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.9...v2.28.10\n\n[2.28.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.8...v2.28.9\n\n[2.28.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.7...v2.28.8\n\n[2.28.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.6...v2.28.7\n\n[2.28.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.5...v2.28.6\n\n[2.28.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.4...v2.28.5\n\n[2.28.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.3...v2.28.4\n\n[2.28.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.2...v2.28.3\n\n[2.28.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.1...v2.28.2\n\n[2.28.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.0...v2.28.1\n\n[2.28.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.15...v2.28.0\n\n[2.27.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.14...v2.27.15\n\n[2.27.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.13...v2.27.14\n\n[2.27.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.12...v2.27.13\n\n[2.27.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.11...v2.27.12\n\n[2.27.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.10...v2.27.11\n\n[2.27.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.9...v2.27.10\n\n[2.27.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.8...v2.27.9\n\n[2.27.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.7...v2.27.8\n\n[2.27.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.6...v2.27.7\n\n[2.27.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.5...v2.27.6\n\n[2.27.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.4...v2.27.5\n\n[2.27.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.3...v2.27.4\n\n[2.27.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.2...v2.27.3\n\n[2.27.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.1...v2.27.2\n\n[2.27.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.0...v2.27.1\n\n[2.27.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.20...v2.27.0\n\n[2.26.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.19...v2.26.20\n\n[2.26.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.18...v2.26.19\n\n[2.26.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.17...v2.26.18\n\n[2.26.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.16...v2.26.17\n\n[2.26.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.15...v2.26.16\n\n[2.26.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.14...v2.26.15\n\n[2.26.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.13...v2.26.14\n\n[2.26.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.12...v2.26.13\n\n[2.26.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.11...v2.26.12\n\n[2.26.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.10...v2.26.11\n\n[2.26.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.9...v2.26.10\n\n[2.26.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.8...v2.26.9\n\n[2.26.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.7...v2.26.8\n\n[2.26.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.6...v2.26.7\n\n[2.26.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.5...v2.26.6\n\n[2.26.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.4...v2.26.5\n\n[2.26.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.3...v2.26.4\n\n[2.26.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.2...v2.26.3\n\n[2.26.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.1...v2.26.2\n\n[2.26.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.0...v2.26.1\n\n[2.26.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.25.11...v2.26.0\n\n[2.25.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.25.10...v2.25.11\n\n[2.25.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.25.9...v2.25.10\n\n[2.25.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.25.8...v2.25.9\n\n[2.25.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.25.7...v2.25.8\n\n[2.25.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.25.6...v2.25.7\n\n[2.25.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.25.5...v2.25.6\n\n[2.25.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.25.4...v2.25.5\n\n[2.25.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.25.3...v2.25.4\n\n[2.25.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.25.2...v2.25.3\n\n[2.25.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.25.1...v2.25.2\n\n[2.25.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.25.0...v2.25.1\n\n[2.25.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.24.4...v2.25.0\n\n[2.24.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.24.3...v2.24.4\n\n[2.24.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.24.2...v2.24.3\n\n[2.24.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.24.1...v2.24.2\n\n[2.24.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.24.0...v2.24.1\n\n[2.24.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.23.9...v2.24.0\n\n[2.23.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.23.8...v2.23.9\n\n[2.23.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.23.7...v2.23.8\n\n[2.23.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.23.6...v2.23.7\n\n[2.23.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.23.5...v2.23.6\n\n[2.23.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.23.4...v2.23.5\n\n[2.23.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.23.3...v2.23.4\n\n[2.23.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.23.2...v2.23.3\n\n[2.23.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.23.1...v2.23.2\n\n[2.23.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.23.0...v2.23.1\n\n[2.23.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.22.10...v2.23.0\n\n[2.22.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.22.9...v2.22.10\n\n[2.22.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.22.8...v2.22.9\n\n[2.22.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.22.7...v2.22.8\n\n[2.22.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.22.6...v2.22.7\n\n[2.22.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.22.5...v2.22.6\n\n[2.22.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.22.4...v2.22.5\n\n[2.22.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.22.3...v2.22.4\n\n[2.22.3]: https://redirect.github.com/taiki-e/install-actio\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - \"before 8am on Monday\" (UTC),\nAutomerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n👻 **Immortal**: This PR will be recreated if closed unmerged. Get\n[config\nhelp](https://redirect.github.com/renovatebot/renovate/discussions) if\nthat's undesired.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0MS45MS4xIiwidXBkYXRlZEluVmVyIjoiNDEuOTcuMTAiLCJ0YXJnZXRCcmFuY2giOiJtYWluIiwibGFiZWxzIjpbImRlcGVuZGVuY2llcyJdfQ==-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2025-09-08T15:07:19Z",
          "tree_id": "38b8ac22a193cf69cfcbb0e9e6cfc0b0e8bdef1f",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/07fcadbe2fd2208f4f69e0bcf59abd649c66ee23"
        },
        "date": 1757344499837,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 736833.3333333334,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 22105000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 22105000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 5.75,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 6.77,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 125.97,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 146.16,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 734666.6666666666,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 22040000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 22040000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-avg",
            "value": 5.67,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 6.63,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 158.08,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 189.26,
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
          "id": "217674cc0db4a57c8d2ec6e33f9d636d1898357a",
          "message": "[query-engine] KQL regex parsing and cleanup (#1082)\n\n## Changes\n\n* Adds regex parsing in KQL\n* Cleans up the KQL pest a bit",
          "timestamp": "2025-09-08T15:14:03Z",
          "tree_id": "07b614c1b9de63e6dfe877df7a2b449058c6179c",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/217674cc0db4a57c8d2ec6e33f9d636d1898357a"
        },
        "date": 1757344931550,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 726666.6666666666,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 21800000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 21800000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 5.66,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 6.69,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 124.34,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 150.52,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 726166.6666666666,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 21785000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 21785000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-avg",
            "value": 5.62,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 6.59,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 154.93,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 187.07,
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
          "id": "c40ffb3cea32d7772039d2315ad969d237c7dd37",
          "message": "Support log `event_name` field (#1090)\n\nCloses #422 \n\n~Opening as draft for now b/c still need to fix/add automated tests.~\n(tests now updated).\n\nI also tested this manually using the following pipeline to confirm that\n`event_name` is being encoded/decoded in both golang and rust\nimplementations by checking debug output\n```\nrust                                 go                                          rust\n[fake_data_gen -> otap_exporter] => [otap receiver -> debug -> otap exporter] => [otap receiver -> debug ...]\n```\n\n---------\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2025-09-08T15:49:27Z",
          "tree_id": "4df75267c969f62ca13824bd1041ba4bae8bbba3",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/c40ffb3cea32d7772039d2315ad969d237c7dd37"
        },
        "date": 1757347132099,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 729333.3333333334,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 21880000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 21880000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 5.55,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 6.57,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 127.91,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 151.01,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 736166.6666666666,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 22085000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 22085000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-avg",
            "value": 5.68,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 6.83,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 173.79,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 207.2,
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
          "id": "c03a3ea488414d4f432a047f4084d5208775dce0",
          "message": "[query-engine] Add support for parsing KQL dynamic expressions (#1092)\n\n## Changes\n\n* Adds support in the KQL parser for the `dynamic` expression",
          "timestamp": "2025-09-08T17:01:12Z",
          "tree_id": "74ba4b38b9ca4ef48aefa7374eaf50600160e418",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/c03a3ea488414d4f432a047f4084d5208775dce0"
        },
        "date": 1757351427182,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 738500,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 22155000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 22155000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 5.61,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 6.63,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 131.58,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 152.42,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 739000,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 22170000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 22170000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-avg",
            "value": 5.7,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 6.75,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 159.48,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 185.37,
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
          "id": "7e91709ca2c9a04e73655fb42f2e97e0d2b9aafb",
          "message": "Add tonic middleware to handle go exporter's grpc-encoding zstdarrow header value (#1094)\n\ncloses: #1068",
          "timestamp": "2025-09-09T15:25:14Z",
          "tree_id": "482948cfd9b6faebf086df00c51ee02f891762ef",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/7e91709ca2c9a04e73655fb42f2e97e0d2b9aafb"
        },
        "date": 1757432007173,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 741500,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 22245000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 22245000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 5.65,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 6.68,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 123.71,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 144.86,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 736500,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 22095000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 22095000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-avg",
            "value": 5.76,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 6.72,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 157.24,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 184.26,
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
          "id": "7430659cb7a66a5755bfb245b724ddfa8e170708",
          "message": "Add configurable arrow IPC compression to otap exporter & align default compression/IPC stream options (#1085)\n\nCloses: #1084 \n\nAdds the ability to set the Arrow IPC compression in the OTAP exporter,\nsimilar to what is configurable in configurable in the golang exporter:\n```yaml\nexporter:\n  config:\n    arrow:\n      payload_compression: zstd / none\n```\n`ztd` is the default, same as golang collector:\n\nhttps://github.com/open-telemetry/opentelemetry-collector-contrib/blob/adbd65ed00ff447a96ce9bdce1320424e5bb420d/exporter/otelarrowexporter/internal/arrow/exporter.go#L62\n\nThis PR also changes the default gRPC compression to zstd, which is also\nwhat is configured by default in the golang collector:\n\nhttps://github.com/open-telemetry/opentelemetry-collector-contrib/blob/adbd65ed00ff447a96ce9bdce1320424e5bb420d/exporter/otelarrowexporter/factory.go#L68-L69\n\nSo by default, we have the \"double compression\".\n\nThis PR also turns on dictionary delta encoding in the IPC stream\nreader. This also aligns with what the go collector configures:\n\nhttps://github.com/open-telemetry/otel-arrow/blob/18996ac489c7ed98f315b803275a3dc112844fc5/go/pkg/otel/arrow_record/producer.go#L403",
          "timestamp": "2025-09-09T15:33:55Z",
          "tree_id": "e26b2129a17c5515be7df5ef141a3b93a1dd19c1",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/7430659cb7a66a5755bfb245b724ddfa8e170708"
        },
        "date": 1757432566392,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 734500,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 22035000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 22035000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 5.67,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 6.75,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 122.2,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 143.2,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 741500,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 22245000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 22245000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-avg",
            "value": 5.75,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 6.71,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 163.91,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 190.74,
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
          "distinct": true,
          "id": "24a0b50130105810ac7359ff63656ab6cca45f49",
          "message": "[otap-dataflow] SignalTypeRouter: internal telemetry (#1097)\n\nSummary:\n•  Add internal telemetry to SignalTypeRouter:\n• Define SignalTypeRouterMetrics (signals_received_,\nsignals_routed_named_, signals_routed_default_, signals_dropped_).\n• Register metrics via PipelineContext (with_pipeline_ctx) and handle\nCollectTelemetry to report snapshots.\n• Increment counters on every PData: detect named vs default port,\nrecord success/failure per signal type.\n• Add unit tests (scoped under tests::telemetry) covering\nlogs/metrics/traces across named/default routing (success and forced\nfailure) using MetricsSystem for collection.\n• Tests only affect test code; production changes are limited to the\nrouter’s telemetry instrumentation and collection path.",
          "timestamp": "2025-09-09T15:56:59Z",
          "tree_id": "deb6b6bc308b54e3b07dc9a5f94880c16ede4aa4",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/24a0b50130105810ac7359ff63656ab6cca45f49"
        },
        "date": 1757433909260,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 729000,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 21870000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 21870000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 5.56,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 6.52,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 123.15,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 141.96,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 740166.6666666666,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 22205000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 22205000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-avg",
            "value": 5.79,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 6.85,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 155.8,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 182.04,
            "unit": "MiB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "129437996+c1ly@users.noreply.github.com",
            "name": "c1ly",
            "username": "c1ly"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "5b0ba7838cf22a4bf820f8d71226f56bd45c5d87",
          "message": "[otap-dataflow] add signal option to the debug processor config (#1093)\n\nUpdate the debug processor to allow the user to select which signals\nshould get displayed\n\n---------\n\nCo-authored-by: Joshua MacDonald <jmacd@users.noreply.github.com>",
          "timestamp": "2025-09-09T19:41:22Z",
          "tree_id": "a73f9ebf538735ff495fe2567154107d3620c98d",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/5b0ba7838cf22a4bf820f8d71226f56bd45c5d87"
        },
        "date": 1757449788331,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 729833.3333333334,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 21895000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 21895000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 5.68,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 6.71,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 118.79,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 138.37,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 732500,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 21975000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 21975000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-avg",
            "value": 5.75,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 6.79,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 166.05,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 193.03,
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
          "id": "b3f8da2cf3bd2952fbbaac8d1c8ae0715906dae3",
          "message": "[otap-dataflow] Attributes processor telemetry (#1066)\n\nAdd internal telemetry reporting for `attributes_processor.rs`\n\n- msgs.consumed: incremented when a PData message is received.\n- msgs.forwarded: incremented after a successful send (both fast path\nwith no actions and after transform).\n- transform.failed: incremented if the transform application returns an\nerror.\n- renamed.entries: exact total added from\ntransform_attributes_with_stats for all targeted payloads in the\nmessage.\n- deleted.entries: exact total added from\ntransform_attributes_with_stats for all targeted payloads in the\nmessage.\n- domains.signal: incremented once per message if apply_to includes\n“signal” (only when actions exist).\n- domains.resource: incremented once per message if apply_to includes\n“resource” (only when actions exist).\n- domains.scope: incremented once per message if apply_to includes\n“scope” (only when actions exist).\n\n---------\n\nCo-authored-by: Joshua MacDonald <jmacd@users.noreply.github.com>",
          "timestamp": "2025-09-10T17:15:41Z",
          "tree_id": "5aa054b3217aa624f5837d220d384bd59ae6a2d8",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/b3f8da2cf3bd2952fbbaac8d1c8ae0715906dae3"
        },
        "date": 1757525053381,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 736500,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 22095000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 22095000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 5.62,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 6.69,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 122.03,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 140.75,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 732833.3333333334,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 21985000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 21985000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-avg",
            "value": 5.65,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 6.83,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 163.03,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 199.07,
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
          "distinct": true,
          "id": "549b3330b14b0ff3ca09a0373f64ff51f3de955f",
          "message": "[otap-dataflow] Config: enforce serde strictness and normalize option casing (#1107)\n\n- Add #[serde(deny_unknown_fields)] to user-facing config structs:\n  - EngineConfig/EngineSettings/TelemetrySettings/HttpAdminSettings\n  - PipelineConfig/PipelineSettings\n  - PipelineGroupConfig/Quota\n  - NodeUserConfig/HyperEdgeConfig\n  - telemetry::Config, state::Config\n  - otap: exporter/receiver/debug/fake_data/syslog_cef/perf configs\n- Normalize enums to canonical snake_case where applicable:\n  - otap::CompressionMethod\n  - otap::otap_exporter::ArrowPayloadCompression\n  - syslog_cef::Protocol\n- Update existing tests to use canonical values (e.g., gzip), and add\nunit tests to assert unknown fields are rejected and only snake_case\nenum variants are accepted.\n\nThis implements PR 1 of #1099: strict parsing and canonical option\ncasing.",
          "timestamp": "2025-09-10T22:30:23Z",
          "tree_id": "d18babd9aa64d26bdd94a0f56f356e559a970d25",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/549b3330b14b0ff3ca09a0373f64ff51f3de955f"
        },
        "date": 1757543934223,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 735000,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 22050000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 22050000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 5.66,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 6.71,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 143.37,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 167.94,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 742666.6666666666,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 22280000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 22280000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-avg",
            "value": 5.66,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 6.9,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 173.73,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 206.71,
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
          "id": "855a751a8b215aa81e204023992a67671add3267",
          "message": "[query-engine] Rename transformations (#1103)\n\n## Changes\n\n* Adds `Move` and `RenameMapKeys` transformations\n* Adds support for KQL `project-rename` expression",
          "timestamp": "2025-09-11T20:26:06Z",
          "tree_id": "6abff33eaf933c7d00a2bf8051a5ef2ded611f3e",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/855a751a8b215aa81e204023992a67671add3267"
        },
        "date": 1757622875897,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 729333.3333333334,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 21880000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 21880000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 5.63,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 6.8,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 123.34,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 145.22,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 736333.3333333334,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 22090000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 22090000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-avg",
            "value": 5.69,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 6.8,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 165.6,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 195.74,
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
          "id": "ef5b9e19c2767b1919b6570b1d63f53530b1e423",
          "message": "Config: RFC-compliant URN validation and canonical plugin URNs (#1113)\n\nThis PR introduces centralized, RFC 8141–based URN validation and\nupdates all plugin URNs to a canonical, consistent form.\n\nHighlights\n• Add config::urn module using the urn crate (RFC 8141 parsing) with\nproject-specific rules\n• Enforce URN validation during node factory registration\n(receiver/processor/exporter)\n• Canonicalize all plugin URNs in code and sample configs (single-colon,\nexplicit kind suffix)\n•  Define clear URN patterns:\n• otel: urn:otel:<family>(:<subfamily>...):<receiver|processor|exporter>\n•  otap: urn:otap:processor:<name>(:<subname>...)\n• Segment policy: lowercase [a-z0-9._-], non-empty segments separated by\n“:”\n•  Improve error messages (e.g., expected suffix vs found suffix)\n• Tests for valid/invalid forms (empty segments, missing family,\nuppercase NSS, percent-encoding, wrong kind)\n\nWhy\n•  Fail fast and consistently on malformed plugin identifiers\n•  Avoid accepting invalid legacy forms (e.g., double colon)\n•  Make plugin types explicit and discoverable\n\nNotes\n•  NID is case-insensitive per RFC (URN/urn, OTEL/otel accepted)\n•  NSS must be lowercase and match [a-z0-9._-]\n•  Reference: RFC 8141 (https://datatracker.ietf.org/doc/html/rfc8141)\n\nFixes one task on #1099",
          "timestamp": "2025-09-12T00:38:19Z",
          "tree_id": "1eadde747ce02a07194fc93e7cbf36227b428f1a",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/ef5b9e19c2767b1919b6570b1d63f53530b1e423"
        },
        "date": 1757638237313,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 736166.6666666666,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 22085000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 22085000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 5.58,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 6.62,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 114.71,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 134.29,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 726166.6666666666,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 21785000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 21785000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-avg",
            "value": 5.61,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 6.68,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 160.78,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 187.93,
            "unit": "MiB"
          }
        ]
      }
    ]
  }
}