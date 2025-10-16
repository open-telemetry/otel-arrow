window.BENCHMARK_DATA = {
  "lastUpdate": 1760634133575,
  "repoUrl": "https://github.com/open-telemetry/otel-arrow",
  "entries": {
    "Benchmark": [
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
          "id": "9ddb78ee31c5182d4d145a3ba5f72d9da2fe024e",
          "message": "[otap-dataflow] BatchProcessor internal telemetry (#1076)\n\nAdds internal telemetry to the batch processor to observe volumes and\nflush decisions without changing behavior.\n\nKey changes\n• Metrics: define OtapBatchProcessorMetrics (moved to\ncrates/otap/src/otap_batch_processor/metrics.rs); update imports.\nProcessor import path unchanged.\n• Wiring: add optional metrics field and from_config_with_metrics;\nregister and inject MetricSet in the factory.\n• Instrumentation: received_rows_{logs,metrics,traces};\nflushes_{size,timer,shutdown}; split_requests;\ndropped_{conversion,empty_records}; batching_errors; dirty_set/cleared\nper signal; timer_flush{performed,skipped} per signal.\n•  Tests: internal telemetry collection test; small encoder test.\n\nNotes\n•  No functional changes to batching or flush logic; telemetry-only.\n\n---------\n\nCo-authored-by: Joshua MacDonald <jmacd@users.noreply.github.com>",
          "timestamp": "2025-09-12T15:13:44Z",
          "tree_id": "940a07ce1f97a67a6408bf24619b06948b7f3b5e",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/9ddb78ee31c5182d4d145a3ba5f72d9da2fe024e"
        },
        "date": 1757690514087,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 745166.6666666666,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 22355000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 22355000,
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
            "value": 6.81,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 128.87,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 150.39,
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
            "value": 5.7,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 6.77,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 161.08,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 189.15,
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
          "id": "8e81c645ee8ef6b679a2c3f402c67966c3c01b2d",
          "message": "fix invalid example configs (#1117)\n\nFixes: #1116\n\nconfirmed that these configs now work:\n```\n$ cargo run -- --pipeline configs/fake-perf.yaml \n ...\nStarting pipeline using all available cores\nStarting Perf Exporter\n...\n```\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2025-09-12T16:05:39Z",
          "tree_id": "8d2409a0e5e294b8e661ab21db81259b4c50fc98",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/8e81c645ee8ef6b679a2c3f402c67966c3c01b2d"
        },
        "date": 1757693674983,
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
            "value": 5.57,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 6.5,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 127.81,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 150.97,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 730500,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 21915000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 21915000,
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
            "value": 158.97,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 194.01,
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
          "id": "9b5e6f9e6a561c3767f1742c8ab3369b6e41d3f1",
          "message": "task: update missed urn for fake_data_generator (#1118)\n\nOne last detail for #1099 - a missed invalid urn\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2025-09-12T16:59:01Z",
          "tree_id": "df531af7552198103b2a75964aef0da20162cc32",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/9b5e6f9e6a561c3767f1742c8ab3369b6e41d3f1"
        },
        "date": 1757696832188,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 735666.6666666666,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 22070000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 22070000,
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
            "value": 6.56,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 125.03,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 153.9,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 740333.3333333334,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 22210000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 22210000,
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
            "value": 6.63,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 163.11,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 192.94,
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
          "id": "3cdc212536e884d5cdef4793c709e67ce0c83254",
          "message": "[otel-arrow-rust] Simplify OtapBatchStore (#1111)\n\n## Changes\n- Simplify `OtapBatchStore` to use `Self::COUNT` in the return type for\n`into_batches` method\n- Remove `shrink` method as it's no longer required\n\n---------\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2025-09-12T17:02:45Z",
          "tree_id": "57c5319cdc160ca6604eed5b4a88683c18dbbda1",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/3cdc212536e884d5cdef4793c709e67ce0c83254"
        },
        "date": 1757697068221,
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
            "value": 6.73,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 116.51,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 138.83,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 736666.6666666666,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 22100000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 22100000,
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
            "value": 166.7,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 194.19,
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
          "id": "cb81a65cd8dfedf0fdd16a119f1ad5a8ecdaae29",
          "message": "fix(deps): update rust crate schemars to v1 (#1089)\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n|---|---|---|---|\n| [schemars](https://graham.cool/schemars/)\n([source](https://redirect.github.com/GREsau/schemars)) | dependencies |\nmajor | `0.9.0` -> `1.0.0` |\n\n---\n\n### Release Notes\n\n<details>\n<summary>GREsau/schemars (schemars)</summary>\n\n###\n[`v1.0.4`](https://redirect.github.com/GREsau/schemars/blob/HEAD/CHANGELOG.md#104---2025-07-06)\n\n[Compare\nSource](https://redirect.github.com/GREsau/schemars/compare/v1.0.3...v1.0.4)\n\n##### Fixed\n\n- Fix `JsonSchema` impl on\n[atomic](https://doc.rust-lang.org/std/sync/atomic/) types being ignored\non non-nightly compilers due to a buggy `cfg` check\n([#&#8203;453](https://redirect.github.com/GREsau/schemars/issues/453))\n- Fix compatibility with minimal dependency versions, e.g. old(-ish)\nversions of `syn`\n([#&#8203;450](https://redirect.github.com/GREsau/schemars/issues/450))\n- Fix derive for empty tuple variants\n([#&#8203;455](https://redirect.github.com/GREsau/schemars/issues/455))\n\n###\n[`v1.0.3`](https://redirect.github.com/GREsau/schemars/blob/HEAD/CHANGELOG.md#103---2025-06-28)\n\n[Compare\nSource](https://redirect.github.com/GREsau/schemars/compare/v1.0.2...v1.0.3)\n\n##### Fixed\n\n- Fix compile error when a doc comment is set on both a `transparent`\n(or newtype) struct and its field\n([#&#8203;446](https://redirect.github.com/GREsau/schemars/issues/446))\n- Fix `json_schema!()` macro compatibility when used from pre-2021 rust\neditions\n([#&#8203;447](https://redirect.github.com/GREsau/schemars/pull/447))\n\n###\n[`v1.0.2`](https://redirect.github.com/GREsau/schemars/blob/HEAD/CHANGELOG.md#102---2025-06-26)\n\n[Compare\nSource](https://redirect.github.com/GREsau/schemars/compare/v1.0.1...v1.0.2)\n\n##### Fixed\n\n- Fix schema properties being incorrectly reordered during serialization\n([#&#8203;444](https://redirect.github.com/GREsau/schemars/issues/444))\n\n###\n[`v1.0.1`](https://redirect.github.com/GREsau/schemars/blob/HEAD/CHANGELOG.md#101---2025-06-24)\n\n[Compare\nSource](https://redirect.github.com/GREsau/schemars/compare/v1.0.0...v1.0.1)\n\n##### Fixed\n\n- Deriving `JsonSchema` with `no_std` broken due to\n`std::borrow::ToOwned` trait not being in scope\n([#&#8203;441](https://redirect.github.com/GREsau/schemars/issues/441))\n\n###\n[`v1.0.0`](https://redirect.github.com/GREsau/schemars/blob/HEAD/CHANGELOG.md#100---2025-06-23)\n\n[Compare\nSource](https://redirect.github.com/GREsau/schemars/compare/v0.9.0...v1.0.0)\n\nThis is a major release with many additions, fixes and changes since 0.8\n(but not many since 0.9). While the basic usage (deriving `JsonSchema`\nand using `schema_for!()` or `SchemaGenerator`) is mostly unchanged, you\nmay wish to consult the [migration\nguide](https://graham.cool/schemars/migrating/) which covers some of the\nmost significant changes.\n\nChanges since 1.0.0-rc.2:\n\n##### Added\n\n- `#[schemars(bound = ...)]` attributes are now used from fields as well\nas containers\n- The\n[`Schema::pointer(...)`](https://docs.rs/schemars/1.0.0/schemars/struct.Schema.html#method.pointer)\nmethod now works when given a JSON pointer in URI Fragment\nrepresentation with a leading `#` character. In particular, this means\nthat you can now lookup a schema from a `$ref` value using that method.\n\n##### Fixed\n\n- Schema names that contain special characters are now correctly encoded\nwhen used inside a `$ref` value\n([#&#8203;436](https://redirect.github.com/GREsau/schemars/pull/436))\n- Optimise type param usage in `SchemaGenerator::subschema_for`,\nreducing LLVM line count and improving compile times\n([#&#8203;439](https://redirect.github.com/GREsau/schemars/pull/439))\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - \"before 8am on Monday\" (UTC),\nAutomerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0MS45MS4xIiwidXBkYXRlZEluVmVyIjoiNDEuOTcuMTAiLCJ0YXJnZXRCcmFuY2giOiJtYWluIiwibGFiZWxzIjpbImRlcGVuZGVuY2llcyJdfQ==-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2025-09-12T17:25:00Z",
          "tree_id": "507873b69196a8f487b7de6357409f6f39492a7c",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/cb81a65cd8dfedf0fdd16a119f1ad5a8ecdaae29"
        },
        "date": 1757698635058,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 738000,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 22140000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 22140000,
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
            "value": 6.82,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 122.85,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 145.15,
            "unit": "MiB"
          },
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
            "value": 5.82,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 6.94,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 171.87,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 202.81,
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
          "id": "ef208f1b0d5bb159e86a60356748226ff9d1f865",
          "message": "Encode OTAP -> OTLP proto bytes directly (#1109)\n\npart of #1095 \ncloses https://github.com/open-telemetry/otel-arrow/issues/1123\n\nThis PR adds an optimized mechanism for encoding OTAP -> OTLP proto\nbytes directly. This implements the encoding only for Logs. Traces &\nMetrics can happen in followup PRs. This PR also integrates this\noptimized mechanism into the OTLP exporter.\n\nIn order to do the encoding in a single pass, we allocate a fixed sized\nlength encoding for some fields and zero-pad the varint. This is not\noptimal from a storage perspective, and we could probably try to\noptimize the size we guess in the future by inspecting the arrow data.\n\nThis PR also doesn't implement direct encoding for Map/Slice type\nattributes, which are CBOR encoded. This can be done in a followup PR.\nFor now, we decode the CBOR to the prost struct and encode that.\n\nSome quick performance testing was performed on a pipeline like\nfollowing:\n- loadgen = 500,000 logs/sec\n- batch size 8000\n- single CPU core per pipeline\n- machine = M4 macbook pro (my laptop)\n```\n                                                                \n      ┌──────────────────────────────────┐                      \n      │ load gen      core 1             │                      \n      │                                  │                      \n      │ ┌────────────┐     ┌───────────┐ │                      \n      │ │ fake data  ┼────►│OTLP       ┼─┼───────┐              \n      │ │ generator  │     │exporter   │ │       │              \n      │ └────────────┘     └───────────┘ │       │              \n      └──────────────────────────────────┘       │              \n                                                 │              \n                                                 │              \n  ┌──────────────────────────────────────────────┘              \n  │                                                             \n  │  ┌──────────────────────────────────────────────┐           \n  │  │    system under test          core 2         │           \n  │  │                                              │           \n  │  │┌────────┐    ┌────────────┐     ┌───────────┐│           \n  │  ││OTLP    │    │  attribute │     │  OTLP     ││           \n  └─►►│Receiver┼───►│  processor ┼─────►  Exporter ┼┼─────► ... \n     │└────────┘    └────────────┘     └───────────┘│           \n     │                                              │           \n     └──────────────────────────────────────────────┘           \n                                                                \n```\nResults - CPU average utilization:\n- `main` 65.1%\n- `albert/1095`44.3%\n- ~20% decrease\n\n---------\n\nCo-authored-by: Utkarsh Umesan Pillai <66651184+utpilla@users.noreply.github.com>\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2025-09-12T17:41:01Z",
          "tree_id": "19fa08ef7e5ff9589a5178d5d7a15d6bb1be30e4",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/ef208f1b0d5bb159e86a60356748226ff9d1f865"
        },
        "date": 1757699371312,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 724500,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 21735000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 21735000,
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
            "value": 6.63,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 127.42,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 156.99,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 728000,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 21840000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 21840000,
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
            "value": 6.57,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 169.12,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 206.22,
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
          "id": "7e81f6a4cad773deb34c4a89de10cd17ed1c28b0",
          "message": "[otap-dataflow] Add OutputMode to the DebugProcessor (#1108)\n\nPart of https://github.com/open-telemetry/otel-arrow/issues/1075\n\nOutputMode determines if the debug processor should output by batches or\nby signal\n\nIn the `batch` mode the debug processor will generate a report given the\nentire batch and then output it to the console\nin the `signal` mode the debug processor will generate a report for each\nindividual signal and then output it to the console",
          "timestamp": "2025-09-12T18:07:58Z",
          "tree_id": "ca9600b13435c8cd401a388c9ba3e4e392e2e405",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/7e81f6a4cad773deb34c4a89de10cd17ed1c28b0"
        },
        "date": 1757700991498,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 736000,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 22080000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 22080000,
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
            "value": 6.73,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 119.18,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 137.69,
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
            "value": 165.36,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 195.69,
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
          "id": "5690c36338ba6be2be22efbe596234b0d99537f5",
          "message": "Fix typo (#1125)\n\n## Changes\n- Rename `ANY_VALUE_BYES_VALUE` to `ANY_VALUE_BYTES_VALUE`",
          "timestamp": "2025-09-12T19:23:27Z",
          "tree_id": "a3b99fd7ae0ade8a7e3dd8731c9ee956d5ed4e95",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/5690c36338ba6be2be22efbe596234b0d99537f5"
        },
        "date": 1757705573010,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 734666.6666666666,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 22040000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 22040000,
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
            "value": 6.64,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 132.47,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 152.68,
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
            "value": 5.78,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 6.78,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 166.65,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 199.52,
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
          "id": "551fab727d01f2eeabcfe6f8e72049480ff68e6b",
          "message": "[otel-arrow-rust] Minor optimization for batch_length method (#1124)\n\n## Changes\n- Minor optimization for Metrics by counting batch length in one pass\ninstead of two\n- Simplify the iterators for counting batch length for Logs and Traces",
          "timestamp": "2025-09-12T20:31:37Z",
          "tree_id": "02fa799f8cf13eed27db429e818abd11cb53668b",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/551fab727d01f2eeabcfe6f8e72049480ff68e6b"
        },
        "date": 1757709634770,
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
            "value": 124.87,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 145.12,
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
            "value": 6.67,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 174.54,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 199.95,
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
          "id": "77995ce00ce72061ad90dbcdbf7db3073367090e",
          "message": "Minor refactoring (#1128)\n\n## Changes\n- Mark methods `const` where applicable\n- Simplify the handling of `ArrowPayloadType::MultivariateMetrics` logic\nwithin the `match` statement to avoid recursive calling of the method\n`child_payload_types`",
          "timestamp": "2025-09-13T06:53:50Z",
          "tree_id": "fcd5619bd2f9cd0858bd29faafdc0eebcf4380c1",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/77995ce00ce72061ad90dbcdbf7db3073367090e"
        },
        "date": 1757746935134,
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
            "value": 5.61,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 6.66,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 121.36,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 142.23,
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
            "value": 5.58,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 6.67,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 168.62,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 200.06,
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
          "id": "58318dcba1952e670dda405cf47e315091dad1fc",
          "message": "chore(deps): update opentelemetry-python monorepo to v1.37.0 (#1132)\n\nThis PR contains the following updates:\n\n| Package | Change | Age | Confidence |\n|---|---|---|---|\n|\n[opentelemetry-exporter-otlp](https://redirect.github.com/open-telemetry/opentelemetry-python)\n| `==1.36.0` -> `==1.37.0` |\n[![age](https://developer.mend.io/api/mc/badges/age/pypi/opentelemetry-exporter-otlp/1.37.0?slim=true)](https://docs.renovatebot.com/merge-confidence/)\n|\n[![confidence](https://developer.mend.io/api/mc/badges/confidence/pypi/opentelemetry-exporter-otlp/1.36.0/1.37.0?slim=true)](https://docs.renovatebot.com/merge-confidence/)\n|\n|\n[opentelemetry-proto](https://redirect.github.com/open-telemetry/opentelemetry-python)\n| `==1.36.0` -> `==1.37.0` |\n[![age](https://developer.mend.io/api/mc/badges/age/pypi/opentelemetry-proto/1.37.0?slim=true)](https://docs.renovatebot.com/merge-confidence/)\n|\n[![confidence](https://developer.mend.io/api/mc/badges/confidence/pypi/opentelemetry-proto/1.36.0/1.37.0?slim=true)](https://docs.renovatebot.com/merge-confidence/)\n|\n|\n[opentelemetry-sdk](https://redirect.github.com/open-telemetry/opentelemetry-python)\n| `==1.36.0` -> `==1.37.0` |\n[![age](https://developer.mend.io/api/mc/badges/age/pypi/opentelemetry-sdk/1.37.0?slim=true)](https://docs.renovatebot.com/merge-confidence/)\n|\n[![confidence](https://developer.mend.io/api/mc/badges/confidence/pypi/opentelemetry-sdk/1.36.0/1.37.0?slim=true)](https://docs.renovatebot.com/merge-confidence/)\n|\n\n---\n\n### Release Notes\n\n<details>\n<summary>open-telemetry/opentelemetry-python\n(opentelemetry-exporter-otlp)</summary>\n\n###\n[`v1.37.0`](https://redirect.github.com/open-telemetry/opentelemetry-python/blob/HEAD/CHANGELOG.md#Version-1370058b0-2025-09-11)\n\n[Compare\nSource](https://redirect.github.com/open-telemetry/opentelemetry-python/compare/v1.36.0...v1.37.0)\n\n- Add experimental composite samplers\n\n([#&#8203;4714](https://redirect.github.com/open-telemetry/opentelemetry-python/pull/4714))\n- Add new environment variables to the SDK\n`OTEL_PYTHON_EXPORTER_OTLP_{HTTP/GRPC}_{METRICS/TRACES/LOGS}_CREDENTIAL_PROVIDER`\nthat can be used to\ninject a `requests.Session` or `grpc.ChannelCredentials` object into\nOTLP exporters created during auto instrumentation\n[#&#8203;4689](https://redirect.github.com/open-telemetry/opentelemetry-python/pull/4689).\n- Filter duplicate logs out of some internal `logger`'s logs on the\nexport logs path that might otherwise endlessly log or cause a recursion\ndepth exceeded issue in cases where logging itself results in an\nexception.\n\n([#&#8203;4695](https://redirect.github.com/open-telemetry/opentelemetry-python/pull/4695)).\n- docs: linked the examples with their github source code location and\nadded Prometheus example\n\n([#&#8203;4728](https://redirect.github.com/open-telemetry/opentelemetry-python/pull/4728))\n- Permit to override default HTTP OTLP exporters headers\n\n([#&#8203;4634](https://redirect.github.com/open-telemetry/opentelemetry-python/pull/4634))\n- semantic-conventions: Bump to 1.37.0\n\n([#&#8203;4731](https://redirect.github.com/open-telemetry/opentelemetry-python/pull/4731))\n- opentelemetry-sdk: fix handling of OTEL\\_ATTRIBUTE\\_COUNT\\_LIMIT in\nlogs\n\n([#&#8203;4677](https://redirect.github.com/open-telemetry/opentelemetry-python/pull/4677))\n- Performance: Cache `importlib_metadata.entry_points`\n\n([#&#8203;4735](https://redirect.github.com/open-telemetry/opentelemetry-python/pull/4735))\n- opentelemetry-sdk: fix calling Logger.emit with an API LogRecord\ninstance\n\n([#&#8203;4741](https://redirect.github.com/open-telemetry/opentelemetry-python/pull/4741))\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - \"before 8am on Monday\" (UTC),\nAutomerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about these\nupdates again.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0MS45Ny4xMCIsInVwZGF0ZWRJblZlciI6IjQxLjk3LjEwIiwidGFyZ2V0QnJhbmNoIjoibWFpbiIsImxhYmVscyI6WyJkZXBlbmRlbmNpZXMiXX0=-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2025-09-15T15:37:05Z",
          "tree_id": "761139bdbfda137e43ca1ab5a72909b57404f609",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/58318dcba1952e670dda405cf47e315091dad1fc"
        },
        "date": 1757951195190,
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
            "value": 5.66,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 6.83,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 120.98,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 142.54,
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
            "value": 162.8,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 194.47,
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
          "id": "ed2e722f0ad2ab34574ba48458ab8ab32e272a87",
          "message": "chore(deps): update dependency pydantic to v2.11.9 (#1130)\n\nComing soon: The Renovate bot (GitHub App) will be renamed to Mend. PRs\nfrom Renovate will soon appear from 'Mend'. Learn more\n[here](https://redirect.github.com/renovatebot/renovate/discussions/37842).\n\nThis PR contains the following updates:\n\n| Package | Change | Age | Confidence |\n|---|---|---|---|\n| [pydantic](https://redirect.github.com/pydantic/pydantic)\n([changelog](https://docs.pydantic.dev/latest/changelog/)) | `==2.11.7`\n-> `==2.11.9` |\n[![age](https://developer.mend.io/api/mc/badges/age/pypi/pydantic/2.11.9?slim=true)](https://docs.renovatebot.com/merge-confidence/)\n|\n[![confidence](https://developer.mend.io/api/mc/badges/confidence/pypi/pydantic/2.11.7/2.11.9?slim=true)](https://docs.renovatebot.com/merge-confidence/)\n|\n\n---\n\n> [!WARNING]\n> Some dependencies could not be looked up. Check the Dependency\nDashboard for more information.\n\n---\n\n### Release Notes\n\n<details>\n<summary>pydantic/pydantic (pydantic)</summary>\n\n###\n[`v2.11.9`](https://redirect.github.com/pydantic/pydantic/releases/tag/v2.11.9):\n2025-09-13\n\n[Compare\nSource](https://redirect.github.com/pydantic/pydantic/compare/v2.11.8...v2.11.9)\n\n##### What's Changed\n\n##### Fixes\n\n- Backport v1.10.23 changes by\n[@&#8203;Viicos](https://redirect.github.com/Viicos)\n\n**Full Changelog**:\n<https://github.com/pydantic/pydantic/compare/v2.11.8...v2.11.9>\n\n###\n[`v2.11.8`](https://redirect.github.com/pydantic/pydantic/releases/tag/v2.11.8):\n2025-09-13\n\n[Compare\nSource](https://redirect.github.com/pydantic/pydantic/compare/v2.11.7...v2.11.8)\n\n#### v2.11.8 (2025-09-13)\n\n##### What's Changed\n\n##### Fixes\n\n- Fix mypy plugin for mypy 1.18 by\n[@&#8203;cdce8p](https://redirect.github.com/cdce8p) in\n[#&#8203;12209](https://redirect.github.com/pydantic/pydantic/pull/12209)\n\n**Full Changelog**:\n<https://github.com/pydantic/pydantic/compare/v2.11.7...v2.11.8>\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - \"before 8am every weekday\" (UTC),\nAutomerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0MS45Ny4xMCIsInVwZGF0ZWRJblZlciI6IjQxLjk3LjEwIiwidGFyZ2V0QnJhbmNoIjoibWFpbiIsImxhYmVscyI6WyJkZXBlbmRlbmNpZXMiXX0=-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2025-09-15T16:02:15Z",
          "tree_id": "e771513008c508d74ecf1d0e62007888775e02a2",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/ed2e722f0ad2ab34574ba48458ab8ab32e272a87"
        },
        "date": 1757952661387,
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
            "value": 126.17,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 147.73,
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
            "value": 5.71,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 6.83,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 162.78,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 190.43,
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
          "id": "89947489a15cdaa4806fda7e810a0e9240474f69",
          "message": "chore(deps): update github workflow dependencies (#1131)\n\nComing soon: The Renovate bot (GitHub App) will be renamed to Mend. PRs\nfrom Renovate will soon appear from 'Mend'. Learn more\n[here](https://redirect.github.com/renovatebot/renovate/discussions/37842).\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n|---|---|---|---|\n|\n[actions/create-github-app-token](https://redirect.github.com/actions/create-github-app-token)\n| action | patch | `v2.1.1` -> `v2.1.4` |\n|\n[github/codeql-action](https://redirect.github.com/github/codeql-action)\n| action | patch | `v3.30.1` -> `v3.30.3` |\n|\n[step-security/harden-runner](https://redirect.github.com/step-security/harden-runner)\n| action | patch | `v2.13.0` -> `v2.13.1` |\n|\n[taiki-e/install-action](https://redirect.github.com/taiki-e/install-action)\n| action | minor | `v2.59.1` -> `v2.61.7` |\n\n---\n\n### Release Notes\n\n<details>\n<summary>actions/create-github-app-token\n(actions/create-github-app-token)</summary>\n\n###\n[`v2.1.4`](https://redirect.github.com/actions/create-github-app-token/releases/tag/v2.1.4)\n\n[Compare\nSource](https://redirect.github.com/actions/create-github-app-token/compare/v2.1.3...v2.1.4)\n\n##### Bug Fixes\n\n- **deps:** bump\n[@&#8203;octokit/auth-app](https://redirect.github.com/octokit/auth-app)\nfrom 7.2.1 to 8.0.1\n([#&#8203;257](https://redirect.github.com/actions/create-github-app-token/issues/257))\n([bef1eaf](https://redirect.github.com/actions/create-github-app-token/commit/bef1eaf1c0ac2b148ee2a0a74c65fbe6db0631f1))\n\n###\n[`v2.1.3`](https://redirect.github.com/actions/create-github-app-token/releases/tag/v2.1.3)\n\n[Compare\nSource](https://redirect.github.com/actions/create-github-app-token/compare/v2.1.2...v2.1.3)\n\n##### Bug Fixes\n\n- **deps:** bump undici from 7.8.0 to 7.10.0 in the\nproduction-dependencies group\n([#&#8203;254](https://redirect.github.com/actions/create-github-app-token/issues/254))\n([f3d5ec2](https://redirect.github.com/actions/create-github-app-token/commit/f3d5ec20739b0cf6f0d52e5a051b65484c378ec9))\n\n###\n[`v2.1.2`](https://redirect.github.com/actions/create-github-app-token/releases/tag/v2.1.2)\n\n[Compare\nSource](https://redirect.github.com/actions/create-github-app-token/compare/v2.1.1...v2.1.2)\n\n##### Bug Fixes\n\n- **deps:** bump\n[@&#8203;octokit/request](https://redirect.github.com/octokit/request)\nfrom 9.2.3 to 10.0.2\n([#&#8203;256](https://redirect.github.com/actions/create-github-app-token/issues/256))\n([5d7307b](https://redirect.github.com/actions/create-github-app-token/commit/5d7307be63501c0070c634b0ae8fec74e8208130))\n\n</details>\n\n<details>\n<summary>github/codeql-action (github/codeql-action)</summary>\n\n###\n[`v3.30.3`](https://redirect.github.com/github/codeql-action/releases/tag/v3.30.3)\n\n[Compare\nSource](https://redirect.github.com/github/codeql-action/compare/v3.30.2...v3.30.3)\n\n### CodeQL Action Changelog\n\nSee the [releases\npage](https://redirect.github.com/github/codeql-action/releases) for the\nrelevant changes to the CodeQL CLI and language packs.\n\n#### 3.30.3 - 10 Sep 2025\n\nNo user facing changes.\n\nSee the full\n[CHANGELOG.md](https://redirect.github.com/github/codeql-action/blob/v3.30.3/CHANGELOG.md)\nfor more information.\n\n###\n[`v3.30.2`](https://redirect.github.com/github/codeql-action/releases/tag/v3.30.2)\n\n[Compare\nSource](https://redirect.github.com/github/codeql-action/compare/v3.30.1...v3.30.2)\n\n### CodeQL Action Changelog\n\nSee the [releases\npage](https://redirect.github.com/github/codeql-action/releases) for the\nrelevant changes to the CodeQL CLI and language packs.\n\n#### 3.30.2 - 09 Sep 2025\n\n- Fixed a bug which could cause language autodetection to fail.\n[#&#8203;3084](https://redirect.github.com/github/codeql-action/pull/3084)\n- Experimental: The `quality-queries` input that was added in `3.29.2`\nas part of an internal experiment is now deprecated and will be removed\nin an upcoming version of the CodeQL Action. It has been superseded by a\nnew `analysis-kinds` input, which is part of the same internal\nexperiment. Do not use this in production as it is subject to change at\nany time.\n[#&#8203;3064](https://redirect.github.com/github/codeql-action/pull/3064)\n\nSee the full\n[CHANGELOG.md](https://redirect.github.com/github/codeql-action/blob/v3.30.2/CHANGELOG.md)\nfor more information.\n\n</details>\n\n<details>\n<summary>step-security/harden-runner\n(step-security/harden-runner)</summary>\n\n###\n[`v2.13.1`](https://redirect.github.com/step-security/harden-runner/releases/tag/v2.13.1)\n\n[Compare\nSource](https://redirect.github.com/step-security/harden-runner/compare/v2.13.0...v2.13.1)\n\n##### What's Changed\n\n- Graceful handling of HTTP errors: Improved error handling when\nfetching Harden Runner policies from the StepSecurity Policy Store API,\nensuring more reliable execution even in case of temporary network/API\nissues.\n\n- Security updates for npm dependencies: Updated vulnerable npm package\ndependencies to the latest secure versions.\n\n- Faster enterprise agent downloads: The enterprise agent is now\ndownloaded from GitHub Releases instead of packages.stepsecurity.io,\nimproving download speed and reliability.\n\n**Full Changelog**:\n<https://github.com/step-security/harden-runner/compare/v2.13.0...v2.13.1>\n\n</details>\n\n<details>\n<summary>taiki-e/install-action (taiki-e/install-action)</summary>\n\n###\n[`v2.61.7`](https://redirect.github.com/taiki-e/install-action/blob/HEAD/CHANGELOG.md#100---2021-12-30)\n\n[Compare\nSource](https://redirect.github.com/taiki-e/install-action/compare/v2.61.6...v2.61.7)\n\nInitial release\n\n[Unreleased]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.7...HEAD\n\n[2.61.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.6...v2.61.7\n\n[2.61.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.5...v2.61.6\n\n[2.61.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.4...v2.61.5\n\n[2.61.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.3...v2.61.4\n\n[2.61.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.2...v2.61.3\n\n[2.61.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.1...v2.61.2\n\n[2.61.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.0...v2.61.1\n\n[2.61.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.60.0...v2.61.0\n\n[2.60.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.59.1...v2.60.0\n\n[2.59.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.59.0...v2.59.1\n\n[2.59.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.33...v2.59.0\n\n[2.58.33]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.32...v2.58.33\n\n[2.58.32]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.31...v2.58.32\n\n[2.58.31]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.30...v2.58.31\n\n[2.58.30]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.29...v2.58.30\n\n[2.58.29]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.28...v2.58.29\n\n[2.58.28]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.27...v2.58.28\n\n[2.58.27]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.26...v2.58.27\n\n[2.58.26]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.25...v2.58.26\n\n[2.58.25]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.24...v2.58.25\n\n[2.58.24]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.23...v2.58.24\n\n[2.58.23]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.22...v2.58.23\n\n[2.58.22]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.21...v2.58.22\n\n[2.58.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.20...v2.58.21\n\n[2.58.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.19...v2.58.20\n\n[2.58.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.18...v2.58.19\n\n[2.58.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.17...v2.58.18\n\n[2.58.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.16...v2.58.17\n\n[2.58.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.15...v2.58.16\n\n[2.58.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.14...v2.58.15\n\n[2.58.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.13...v2.58.14\n\n[2.58.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.12...v2.58.13\n\n[2.58.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.11...v2.58.12\n\n[2.58.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.10...v2.58.11\n\n[2.58.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.9...v2.58.10\n\n[2.58.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.8...v2.58.9\n\n[2.58.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.7...v2.58.8\n\n[2.58.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.6...v2.58.7\n\n[2.58.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.5...v2.58.6\n\n[2.58.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.4...v2.58.5\n\n[2.58.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.3...v2.58.4\n\n[2.58.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.2...v2.58.3\n\n[2.58.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.1...v2.58.2\n\n[2.58.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.0...v2.58.1\n\n[2.58.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.8...v2.58.0\n\n[2.57.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.7...v2.57.8\n\n[2.57.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.6...v2.57.7\n\n[2.57.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.5...v2.57.6\n\n[2.57.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.4...v2.57.5\n\n[2.57.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.3...v2.57.4\n\n[2.57.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.2...v2.57.3\n\n[2.57.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.1...v2.57.2\n\n[2.57.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.0...v2.57.1\n\n[2.57.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.24...v2.57.0\n\n[2.56.24]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.23...v2.56.24\n\n[2.56.23]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.22...v2.56.23\n\n[2.56.22]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.21...v2.56.22\n\n[2.56.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.20...v2.56.21\n\n[2.56.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.19...v2.56.20\n\n[2.56.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.18...v2.56.19\n\n[2.56.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.17...v2.56.18\n\n[2.56.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.16...v2.56.17\n\n[2.56.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.15...v2.56.16\n\n[2.56.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.14...v2.56.15\n\n[2.56.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.13...v2.56.14\n\n[2.56.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.12...v2.56.13\n\n[2.56.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.11...v2.56.12\n\n[2.56.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.10...v2.56.11\n\n[2.56.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.9...v2.56.10\n\n[2.56.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.8...v2.56.9\n\n[2.56.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.7...v2.56.8\n\n[2.56.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.6...v2.56.7\n\n[2.56.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.5...v2.56.6\n\n[2.56.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.4...v2.56.5\n\n[2.56.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.3...v2.56.4\n\n[2.56.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.2...v2.56.3\n\n[2.56.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.1...v2.56.2\n\n[2.56.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.0...v2.56.1\n\n[2.56.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.55.4...v2.56.0\n\n[2.55.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.55.3...v2.55.4\n\n[2.55.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.55.2...v2.55.3\n\n[2.55.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.55.1...v2.55.2\n\n[2.55.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.55.0...v2.55.1\n\n[2.55.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.54.3...v2.55.0\n\n[2.54.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.54.2...v2.54.3\n\n[2.54.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.54.1...v2.54.2\n\n[2.54.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.54.0...v2.54.1\n\n[2.54.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.53.2...v2.54.0\n\n[2.53.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.53.1...v2.53.2\n\n[2.53.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.53.0...v2.53.1\n\n[2.53.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.8...v2.53.0\n\n[2.52.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.7...v2.52.8\n\n[2.52.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.6...v2.52.7\n\n[2.52.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.5...v2.52.6\n\n[2.52.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.4...v2.52.5\n\n[2.52.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.3...v2.52.4\n\n[2.52.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.2...v2.52.3\n\n[2.52.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.1...v2.52.2\n\n[2.52.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.0...v2.52.1\n\n[2.52.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.51.3...v2.52.0\n\n[2.51.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.51.2...v2.51.3\n\n[2.51.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.51.1...v2.51.2\n\n[2.51.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.51.0...v2.51.1\n\n[2.51.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.10...v2.51.0\n\n[2.50.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.9...v2.50.10\n\n[2.50.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.8...v2.50.9\n\n[2.50.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.7...v2.50.8\n\n[2.50.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.6...v2.50.7\n\n[2.50.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.5...v2.50.6\n\n[2.50.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.4...v2.50.5\n\n[2.50.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.3...v2.50.4\n\n[2.50.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.2...v2.50.3\n\n[2.50.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.1...v2.50.2\n\n[2.50.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.0...v2.50.1\n\n[2.50.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.50...v2.50.0\n\n[2.49.50]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.49...v2.49.50\n\n[2.49.49]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.48...v2.49.49\n\n[2.49.48]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.47...v2.49.48\n\n[2.49.47]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.46...v2.49.47\n\n[2.49.46]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.45...v2.49.46\n\n[2.49.45]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.44...v2.49.45\n\n[2.49.44]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.43...v2.49.44\n\n[2.49.43]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.42...v2.49.43\n\n[2.49.42]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.41...v2.49.42\n\n[2.49.41]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.40...v2.49.41\n\n[2.49.40]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.39...v2.49.40\n\n[2.49.39]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.38...v2.49.39\n\n[2.49.38]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.37...v2.49.38\n\n[2.49.37]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.36...v2.49.37\n\n[2.49.36]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.35...v2.49.36\n\n[2.49.35]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.34...v2.49.35\n\n[2.49.34]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.33...v2.49.34\n\n[2.49.33]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.32...v2.49.33\n\n[2.49.32]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.31...v2.49.32\n\n[2.49.31]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.30...v2.49.31\n\n[2.49.30]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.29...v2.49.30\n\n[2.49.29]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.28...v2.49.29\n\n[2.49.28]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.27...v2.49.28\n\n[2.49.27]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.26...v2.49.27\n\n[2.49.26]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.25...v2.49.26\n\n[2.49.25]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.24...v2.49.25\n\n[2.49.24]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.23...v2.49.24\n\n[2.49.23]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.22...v2.49.23\n\n[2.49.22]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.21...v2.49.22\n\n[2.49.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.20...v2.49.21\n\n[2.49.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.19...v2.49.20\n\n[2.49.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.18...v2.49.19\n\n[2.49.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.17...v2.49.18\n\n[2.49.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.16...v2.49.17\n\n[2.49.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.15...v2.49.16\n\n[2.49.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.14...v2.49.15\n\n[2.49.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.13...v2.49.14\n\n[2.49.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.12...v2.49.13\n\n[2.49.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.11...v2.49.12\n\n[2.49.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.10...v2.49.11\n\n[2.49.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.9...v2.49.10\n\n[2.49.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.8...v2.49.9\n\n[2.49.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.7...v2.49.8\n\n[2.49.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.6...v2.49.7\n\n[2.49.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.5...v2.49.6\n\n[2.49.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.4...v2.49.5\n\n[2.49.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.3...v2.49.4\n\n[2.49.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.2...v2.49.3\n\n[2.49.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.1...v2.49.2\n\n[2.49.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.0...v2.49.1\n\n[2.49.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.22...v2.49.0\n\n[2.48.22]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.21...v2.48.22\n\n[2.48.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.20...v2.48.21\n\n[2.48.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.19...v2.48.20\n\n[2.48.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.18...v2.48.19\n\n[2.48.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.17...v2.48.18\n\n[2.48.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.16...v2.48.17\n\n[2.48.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.15...v2.48.16\n\n[2.48.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.14...v2.48.15\n\n[2.48.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.13...v2.48.14\n\n[2.48.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.12...v2.48.13\n\n[2.48.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.11...v2.48.12\n\n[2.48.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.10...v2.48.11\n\n[2.48.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.9...v2.48.10\n\n[2.48.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.8...v2.48.9\n\n[2.48.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.7...v2.48.8\n\n[2.48.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.6...v2.48.7\n\n[2.48.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.5...v2.48.6\n\n[2.48.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.4...v2.48.5\n\n[2.48.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.3...v2.48.4\n\n[2.48.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.2...v2.48.3\n\n[2.48.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.1...v2.48.2\n\n[2.48.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.0...v2.48.1\n\n[2.48.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.32...v2.48.0\n\n[2.47.32]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.31...v2.47.32\n\n[2.47.31]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.30...v2.47.31\n\n[2.47.30]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.29...v2.47.30\n\n[2.47.29]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.28...v2.47.29\n\n[2.47.28]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.27...v2.47.28\n\n[2.47.27]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.26...v2.47.27\n\n[2.47.26]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.25...v2.47.26\n\n[2.47.25]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.24...v2.47.25\n\n[2.47.24]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.23...v2.47.24\n\n[2.47.23]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.22...v2.47.23\n\n[2.47.22]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.21...v2.47.22\n\n[2.47.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.20...v2.47.21\n\n[2.47.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.19...v2.47.20\n\n[2.47.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.18...v2.47.19\n\n[2.47.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.17...v2.47.18\n\n[2.47.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.16...v2.47.17\n\n[2.47.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.15...v2.47.16\n\n[2.47.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.14...v2.47.15\n\n[2.47.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.13...v2.47.14\n\n[2.47.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.12...v2.47.13\n\n[2.47.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.11...v2.47.12\n\n[2.47.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.10...v2.47.11\n\n[2.47.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.9...v2.47.10\n\n[2.47.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.8...v2.47.9\n\n[2.47.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.7...v2.47.8\n\n[2.47.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.6...v2.47.7\n\n[2.47.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.5...v2.47.6\n\n[2.47.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.4...v2.47.5\n\n[2.47.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.3...v2.47.4\n\n[2.47.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.2...v2.47.3\n\n[2.47.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.1...v2.47.2\n\n[2.47.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.0...v2.47.1\n\n[2.47.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.20...v2.47.0\n\n[2.46.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.19...v2.46.20\n\n[2.46.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.18...v2.46.19\n\n[2.46.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.17...v2.46.18\n\n[2.46.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.16...v2.46.17\n\n[2.46.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.15...v2.46.16\n\n[2.46.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.14...v2.46.15\n\n[2.46.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.13...v2.46.14\n\n[2.46.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.12...v2.46.13\n\n[2.46.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.11...v2.46.12\n\n[2.46.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.10...v2.46.11\n\n[2.46.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.9...v2.46.10\n\n[2.46.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.8...v2.46.9\n\n[2.46.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.7...v2.46.8\n\n[2.46.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.6...v2.46.7\n\n[2.46.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.5...v2.46.6\n\n[2.46.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.4...v2.46.5\n\n[2.46.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.3...v2.46.4\n\n[2.46.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.2...v2.46.3\n\n[2.46.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.1...v2.46.2\n\n[2.46.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.0...v2.46.1\n\n[2.46.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.15...v2.46.0\n\n[2.45.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.14...v2.45.15\n\n[2.45.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.13...v2.45.14\n\n[2.45.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.12...v2.45.13\n\n[2.45.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.11...v2.45.12\n\n[2.45.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.10...v2.45.11\n\n[2.45.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.9...v2.45.10\n\n[2.45.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.8...v2.45.9\n\n[2.45.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.7...v2.45.8\n\n[2.45.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.6...v2.45.7\n\n[2.45.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.5...v2.45.6\n\n[2.45.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.4...v2.45.5\n\n[2.45.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.3...v2.45.4\n\n[2.45.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.2...v2.45.3\n\n[2.45.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.1...v2.45.2\n\n[2.45.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.0...v2.45.1\n\n[2.45.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.72...v2.45.0\n\n[2.44.72]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.71...v2.44.72\n\n[2.44.71]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.70...v2.44.71\n\n[2.44.70]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.69...v2.44.70\n\n[2.44.69]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.68...v2.44.69\n\n[2.44.68]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.67...v2.44.68\n\n[2.44.67]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.66...v2.44.67\n\n[2.44.66]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.65...v2.44.66\n\n[2.44.65]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.64...v2.44.65\n\n[2.44.64]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.63...v2.44.64\n\n[2.44.63]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.62...v2.44.63\n\n[2.44.62]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.61...v2.44.62\n\n[2.44.61]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.60...v2.44.61\n\n[2.44.60]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.59...v2.44.60\n\n[2.44.59]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.58...v2.44.59\n\n[2.44.58]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.57...v2.44.58\n\n[2.44.57]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.56...v2.44.57\n\n[2.44.56]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.55...v2.44.56\n\n[2.44.55]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.54...v2.44.55\n\n[2.44.54]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.53...v2.44.54\n\n[2.44.53]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.52...v2.44.53\n\n[2.44.52]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.51...v2.44.52\n\n[2.44.51]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.50...v2.44.51\n\n[2.44.50]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.49...v2.44.50\n\n[2.44.49]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.48...v2.44.49\n\n[2.44.48]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.47...v2.44.48\n\n[2.44.47]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.46...v2.44.47\n\n[2.44.46]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.45...v2.44.46\n\n[2.44.45]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.44...v2.44.45\n\n[2.44.44]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.43...v2.44.44\n\n[2.44.43]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.42...v2.44.43\n\n[2.44.42]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.41...v2.44.42\n\n[2.44.41]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.40...v2.44.41\n\n[2.44.40]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.39...v2.44.40\n\n[2.44.39]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.38...v2.44.39\n\n[2.44.38]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.37...v2.44.38\n\n[2.44.37]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.36...v2.44.37\n\n[2.44.36]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.35...v2.44.36\n\n[2.44.35]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.34...v2.44.35\n\n[2.44.34]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.33...v2.44.34\n\n[2.44.33]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.32...v2.44.33\n\n[2.44.32]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.31...v2.44.32\n\n[2.44.31]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.30...v2.44.31\n\n[2.44.30]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.29...v2.44.30\n\n[2.44.29]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.28...v2.44.29\n\n[2.44.28]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.27...v2.44.28\n\n[2.44.27]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.26...v2.44.27\n\n[2.44.26]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.25...v2.44.26\n\n[2.44.25]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.24...v2.44.25\n\n[2.44.24]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.23...v2.44.24\n\n[2.44.23]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.22...v2.44.23\n\n[2.44.22]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.21...v2.44.22\n\n[2.44.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.20...v2.44.21\n\n[2.44.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.19...v2.44.20\n\n[2.44.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.18...v2.44.19\n\n[2.44.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.17...v2.44.18\n\n[2.44.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.16...v2.44.17\n\n[2.44.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.15...v2.44.16\n\n[2.44.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.14...v2.44.15\n\n[2.44.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.13...v2.44.14\n\n[2.44.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.12...v2.44.13\n\n[2.44.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.11...v2.44.12\n\n[2.44.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.10...v2.44.11\n\n[2.44.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.9...v2.44.10\n\n[2.44.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.8...v2.44.9\n\n[2.44.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.7...v2.44.8\n\n[2.44.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.6...v2.44.7\n\n[2.44.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.5...v2.44.6\n\n[2.44.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.4...v2.44.5\n\n[2.44.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.3...v2.44.4\n\n[2.44.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.2...v2.44.3\n\n[2.44.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.1...v2.44.2\n\n[2.44.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.0...v2.44.1\n\n[2.44.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.43.7...v2.44.0\n\n[2.43.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.43.6...v2.43.7\n\n[2.43.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.43.5...v2.43.6\n\n[2.43.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.43.4...v2.43.5\n\n[2.43.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.43.3...v2.43.4\n\n[2.43.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.43.2...v2.43.3\n\n[2.43.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.43.1...v2.43.2\n\n[2.43.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.43.0...v2.43.1\n\n[2.43.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.42...v2.43.0\n\n[2.42.42]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.41...v2.42.42\n\n[2.42.41]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.40...v2.42.41\n\n[2.42.40]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.39...v2.42.40\n\n[2.42.39]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.38...v2.42.39\n\n[2.42.38]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.37...v2.42.38\n\n[2.42.37]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.36...v2.42.37\n\n[2.42.36]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.35...v2.42.36\n\n[2.42.35]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.34...v2.42.35\n\n[2.42.34]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.33...v2.42.34\n\n[2.42.33]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.32...v2.42.33\n\n[2.42.32]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.31...v2.42.32\n\n[2.42.31]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.30...v2.42.31\n\n[2.42.30]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.29...v2.42.30\n\n[2.42.29]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.28...v2.42.29\n\n[2.42.28]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.27...v2.42.28\n\n[2.42.27]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.26...v2.42.27\n\n[2.42.26]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.25...v2.42.26\n\n[2.42.25]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.24...v2.42.25\n\n[2.42.24]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.23...v2.42.24\n\n[2.42.23]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.22...v2.42.23\n\n[2.42.22]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.21...v2.42.22\n\n[2.42.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.20...v2.42.21\n\n[2.42.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.19...v2.42.20\n\n[2.42.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.18...v2.42.19\n\n[2.42.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.17...v2.42.18\n\n[2.42.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.16...v2.42.17\n\n[2.42.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.15...v2.42.16\n\n[2.42.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.14...v2.42.15\n\n[2.42.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.13...v2.42.14\n\n[2.42.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.12...v2.42.13\n\n[2.42.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.11...v2.42.12\n\n[2.42.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.10...v2.42.11\n\n[2.42.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.9...v2.42.10\n\n[2.42.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.8...v2.42.9\n\n[2.42.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.7...v2.42.8\n\n[2.42.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.6...v2.42.7\n\n[2.42.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.5...v2.42.6\n\n[2.42.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.4...v2.42.5\n\n[2.42.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.3...v2.42.4\n\n[2.42.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.2...v2.42.3\n\n[2.42.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.1...v2.42.2\n\n[2.42.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.0...v2.42.1\n\n[2.42.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.18...v2.42.0\n\n[2.41.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.17...v2.41.18\n\n[2.41.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.16...v2.41.17\n\n[2.41.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.15...v2.41.16\n\n[2.41.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.14...v2.41.15\n\n[2.41.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.13...v2.41.14\n\n[2.41.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.12...v2.41.13\n\n[2.41.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.11...v2.41.12\n\n[2.41.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.10...v2.41.11\n\n[2.41.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.9...v2.41.10\n\n[2.41.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.8...v2.41.9\n\n[2.41.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.7...v2.41.8\n\n[2.41.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.6...v2.41.7\n\n[2.41.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.5...v2.41.6\n\n[2.41.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.4...v2.41.5\n\n[2.41.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.3...v2.41.4\n\n[2.41.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.2...v2.41.3\n\n[2.41.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.1...v2.41.2\n\n[2.41.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.0...v2.41.1\n\n[2.41.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.40.2...v2.41.0\n\n[2.40.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.40.1...v2.40.2\n\n[2.40.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.40.0...v2.40.1\n\n[2.40.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.39.2...v2.40.0\n\n[2.39.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.39.1...v2.39.2\n\n[2.39.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.39.0...v2.39.1\n\n[2.39.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.38.7...v2.39.0\n\n[2.38.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.38.6...v2.38.7\n\n[2.38.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.38.5...v2.38.6\n\n[2.38.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.38.4...v2.38.5\n\n[2.38.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.38.3...v2.38.4\n\n[2.38.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.38.2...v2.38.3\n\n[2.38.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.38.1...v2.38.2\n\n[2.38.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.38.0...v2.38.1\n\n[2.38.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.37.0...v2.38.0\n\n[2.37.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.36.0...v2.37.0\n\n[2.36.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.35.0...v2.36.0\n\n[2.35.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.34.3...v2.35.0\n\n[2.34.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.34.2...v2.34.3\n\n[2.34.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.34.1...v2.34.2\n\n[2.34.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.34.0...v2.34.1\n\n[2.34.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.36...v2.34.0\n\n[2.33.36]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.35...v2.33.36\n\n[2.33.35]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.34...v2.33.35\n\n[2.33.34]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.33...v2.33.34\n\n[2.33.33]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.32...v2.33.33\n\n[2.33.32]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.31...v2.33.32\n\n[2.33.31]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.30...v2.33.31\n\n[2.33.30]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.29...v2.33.30\n\n[2.33.29]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.28...v2.33.29\n\n[2.33.28]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.27...v2.33.28\n\n[2.33.27]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.26...v2.33.27\n\n[2.33.26]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.25...v2.33.26\n\n[2.33.25]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.24...v2.33.25\n\n[2.33.24]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.23...v2.33.24\n\n[2.33.23]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.22...v2.33.23\n\n[2.33.22]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.21...v2.33.22\n\n[2.33.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.20...v2.33.21\n\n[2.33.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.19...v2.33.20\n\n[2.33.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.18...v2.33.19\n\n[2.33.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.17...v2.33.18\n\n[2.33.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.16...v2.33.17\n\n[2.33.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.15...v2.33.16\n\n[2.33.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.14...v2.33.15\n\n[2.33.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.13...v2.33.14\n\n[2.33.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.12...v2.33.13\n\n[2.33.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.11...v2.33.12\n\n[2.33.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.10...v2.33.11\n\n[2.33.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.9...v2.33.10\n\n[2.33.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.8...v2.33.9\n\n[2.33.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.7...v2.33.8\n\n[2.33.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.6...v2.33.7\n\n[2.33.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.5...v2.33.6\n\n[2.33.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.4...v2.33.5\n\n[2.33.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.3...v2.33.4\n\n[2.33.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.2...v2.33.3\n\n[2.33.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.1...v2.33.2\n\n[2.33.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.0...v2.33.1\n\n[2.33.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.20...v2.33.0\n\n[2.32.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.19...v2.32.20\n\n[2.32.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.18...v2.32.19\n\n[2.32.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.17...v2.32.18\n\n[2.32.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.16...v2.32.17\n\n[2.32.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.15...v2.32.16\n\n[2.32.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.14...v2.32.15\n\n[2.32.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.13...v2.32.14\n\n[2.32.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.12...v2.32.13\n\n[2.32.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.11...v2.32.12\n\n[2.32.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.10...v2.32.11\n\n[2.32.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.9...v2.32.10\n\n[2.32.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.8...v2.32.9\n\n[2.32.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.7...v2.32.8\n\n[2.32.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.6...v2.32.7\n\n[2.32.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.5...v2.32.6\n\n[2.32.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.4...v2.32.5\n\n[2.32.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.3...v2.32.4\n\n[2.32.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.2...v2.32.3\n\n[2.32.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.1...v2.32.2\n\n[2.32.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.0...v2.32.1\n\n[2.32.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.31.3...v2.32.0\n\n[2.31.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.31.2...v2.31.3\n\n[2.31.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.31.1...v2.31.2\n\n[2.31.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.31.0...v2.31.1\n\n[2.31.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.30.0...v2.31.0\n\n[2.30.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.29.8...v2.30.0\n\n[2.29.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.29.7...v2.29.8\n\n[2.29.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.29.6...v2.29.7\n\n[2.29.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.29.5...v2.29.6\n\n[2.29.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.29.4...v2.29.5\n\n[2.29.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.29.3...v2.29.4\n\n[2.29.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.29.2...v2.29.3\n\n[2.29.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.29.1...v2.29.2\n\n[2.29.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.29.0...v2.29.1\n\n[2.29.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.16...v2.29.0\n\n[2.28.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.15...v2.28.16\n\n[2.28.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.14...v2.28.15\n\n[2.28.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.13...v2.28.14\n\n[2.28.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.12...v2.28.13\n\n[2.28.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.11...v2.28.12\n\n[2.28.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.10...v2.28.11\n\n[2.28.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.9...v2.28.10\n\n[2.28.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.8...v2.28.9\n\n[2.28.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.7...v2.28.8\n\n[2.28.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.6...v2.28.7\n\n[2.28.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.5...v2.28.6\n\n[2.28.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.4...v2.28.5\n\n[2.28.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.3...v2.28.4\n\n[2.28.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.2...v2.28.3\n\n[2.28.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.1...v2.28.2\n\n[2.28.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.0...v2.28.1\n\n[2.28.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.15...v2.28.0\n\n[2.27.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.14...v2.27.15\n\n[2.27.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.13...v2.27.14\n\n[2.27.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.12...v2.27.13\n\n[2.27.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.11...v2.27.12\n\n[2.27.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.10...v2.27.11\n\n[2.27.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.9...v2.27.10\n\n[2.27.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.8...v2.27.9\n\n[2.27.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.7...v2.27.8\n\n[2.27.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.6...v2.27.7\n\n[2.27.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.5...v2.27.6\n\n[2.27.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.4...v2.27.5\n\n[2.27.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.3...v2.27.4\n\n[2.27.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.2...v2.27.3\n\n[2.27.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.1...v2.27.2\n\n[2.27.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.0...v2.27.1\n\n[2.27.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.20...v2.27.0\n\n[2.26.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.19...v2.26.20\n\n[2.26.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.18...v2.26.19\n\n[2.26.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.17...v2.26.18\n\n[2.26.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.16...v2.26.17\n\n[2.26.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.15...v2.26.16\n\n[2.26.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.14...v2.26.15\n\n[2.26.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.13...v2.26.14\n\n[2.26.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.12...v2.26.13\n\n[2.26.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.11...v2.26.12\n\n[2.26.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.10...v2.26.11\n\n[2.26.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.9...v2.26.10\n\n[2.26.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.8...v2.26.9\n\n[2.26.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.7...v2.26.8\n\n[2.26.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.6...v2.26.7\n\n[2.26.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.5...v2.26.6\n\n[2.26.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.4...v2.26.5\n\n[2.26.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.3...v2.26.4\n\n[2.26.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.2...v2.26.3\n\n[2.26.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.1...v2.26.2\n\n[2.26.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.0...v2.26.1\n\n[2.26.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.25.11...v2.26.0\n\n[2.25.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.25.10...v2.25.11\n\n[2.25.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.25.9...v2.25.10\n\n[2.25.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.25.8...v2.25.9\n\n[2.25.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.25.7...v2.25.8\n\n[2.25.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.25.6...v2.25.7\n\n[2.25.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.25.5...v2.25.6\n\n[2.25.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.25.4...v2.25.5\n\n[2.25.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.25.3...v2.25.4\n\n[2.25.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.25.2...v2.25.3\n\n[2.25.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.25.1...v2.25.2\n\n[2.25.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.25.0...v2.25.1\n\n[2.25.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.24.4...v2.25.0\n\n[2.24.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.24.3...v2.24.4\n\n[2.24.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.24.2...v2.24.3\n\n[2.24.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.24.1...v2.24.2\n\n[2.24.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.24.0...v2.24.1\n\n[2.24.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.23.9...v2.24.0\n\n[2.23.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.23.8...v2.23.9\n\n[2.23.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.23.7...v2.23.8\n\n[2.23.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.23.6...v2.23.7\n\n[2.23.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.23.5...v2.23.6\n\n[2.23.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.23.4...v2.23.5\n\n[2.23.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.23.3...v2.23.4\n\n[2.23.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.23.2...v2.23.3\n\n[2.23.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.23.1...v2.23.2\n\n[2.23.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.23.0...v2.23.1\n\n[2.23.0]: https://redirect.github\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - \"before 8am on Monday\" (UTC),\nAutomerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n👻 **Immortal**: This PR will be recreated if closed unmerged. Get\n[config\nhelp](https://redirect.github.com/renovatebot/renovate/discussions) if\nthat's undesired.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0MS45Ny4xMCIsInVwZGF0ZWRJblZlciI6IjQxLjk3LjEwIiwidGFyZ2V0QnJhbmNoIjoibWFpbiIsImxhYmVscyI6WyJkZXBlbmRlbmNpZXMiXX0=-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2025-09-16T15:37:18Z",
          "tree_id": "5e364f8066220aae71f890b6cd25f5fb960439d5",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/89947489a15cdaa4806fda7e810a0e9240474f69"
        },
        "date": 1758037594853,
        "tool": "customSmallerIsBetter",
        "benches": [
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
            "value": 6.61,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 123.77,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 140.4,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 727500,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 21825000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 21825000,
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
            "value": 162.31,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 194.1,
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
          "id": "d922edb95730e163b96d2794b67a97fd5abc2cf8",
          "message": "Encode OTAP -> OTLP proto bytes directly for Traces (#1138)\n\ncloses: #1119 \n\nContinues the work started in #1109 traces\n\n---------\n\nCo-authored-by: Utkarsh Umesan Pillai <66651184+utpilla@users.noreply.github.com>",
          "timestamp": "2025-09-16T23:04:45Z",
          "tree_id": "8615a7687ef5a6b23f957d2991b29bd26587e528",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/d922edb95730e163b96d2794b67a97fd5abc2cf8"
        },
        "date": 1758064392155,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 732666.6666666666,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 21980000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 21980000,
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
            "value": 6.81,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 128.16,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 150.36,
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
            "value": 5.76,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 6.78,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 160.84,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 200.09,
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
          "id": "9474f126091279e97e78620b47edeccd98fc684b",
          "message": "Debug processor internal telemetry (#1067)\n\nCollect the following metrics from the debug processor\n\n```rust\n#[metric_set(name = \"debug.processor.pdata.metrics\")]\n#[derive(Debug, Default, Clone)]\npub struct DebugPdataMetrics {\n    /// Number of logs seen\n    #[metric(unit = \"{log}\")]\n    pub logs: Counter<u64>,\n    /// Number of events (structured logs) seen\n    #[metric(unit = \"{event}\")]\n    pub events: Counter<u64>,\n    /// Number of spans seen\n    #[metric(unit = \"{span}\")]\n    pub spans: Counter<u64>,\n    /// number of span links seen\n    #[metric(unit = \"{link}\")]\n    pub span_links: Counter<u64>,\n    /// number of span events (structured logs) seen\n    #[metric(unit = \"{event}\")]\n    pub span_events: Counter<u64>,\n    /// Number of metrics seen\n    #[metric(unit = \"{metric}\")]\n    pub metrics: Counter<u64>,\n    /// number of metric datapoints seen\n    #[metric(unit = \"{datapoint}\")]\n    pub metric_datapoints: Counter<u64>,\n}\n```\n\n---------\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2025-09-17T17:50:12Z",
          "tree_id": "f613e893223879fbc1dded0d935f8bf379efd7ae",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/9474f126091279e97e78620b47edeccd98fc684b"
        },
        "date": 1758131972619,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 733166.6666666666,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 21995000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 21995000,
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
            "value": 6.59,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 126.04,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 147.69,
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
            "value": 5.84,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 7.1,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 164.82,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 194.5,
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
          "id": "97c3a5aa907064edc007e6735fed9640fd74120c",
          "message": "Encode OTAP -> OTLP proto bytes directly for Metrics (#1143)\n\ncloses: #1120 \n\nContinues the work started in\nhttps://github.com/open-telemetry/otel-arrow/pull/1109 metrics\n\nThis also \n- removes the last of the code that computed OTLP structs directly from\nOTAP, including the `otel_arrow_rust::otlp::store::AttributeStore`.\n- fixes a bug in materializing parent IDs where we didn't handle dict\nencoded parent IDs, which can be present in some attributes where\nparent_id is u32\n\n---------\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>\nCo-authored-by: Laurent Quérel <laurent.querel@gmail.com>",
          "timestamp": "2025-09-17T20:45:53Z",
          "tree_id": "157bb634577884496c0928b293afc4ac67a9cb55",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/97c3a5aa907064edc007e6735fed9640fd74120c"
        },
        "date": 1758142460637,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 739666.6666666666,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 22190000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 22190000,
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
            "value": 6.57,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 128.99,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 149.84,
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
            "value": 5.71,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 6.94,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 161.88,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 186.26,
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
          "id": "b88ba7cb5dc416a54b66738829e2197701c687c2",
          "message": "[query-engine] Clean up scalar expression parsing in KQL parser (#1146)\n\n## Changes\n\n* Clean up the `parse_scalar_expression` method in KQL parser by pushing\ngroup names into the PEST tree",
          "timestamp": "2025-09-17T21:27:50Z",
          "tree_id": "b29a76bd599d175a06455680f4dd194f1a20b4a7",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/b88ba7cb5dc416a54b66738829e2197701c687c2"
        },
        "date": 1758145002441,
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
            "value": 5.57,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 6.56,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 126.19,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 151.55,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 743500,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 22305000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 22305000,
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
            "value": 6.93,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 161.38,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 191.66,
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
          "id": "eaf81aabee7eaa80d4ffe78a59d9bfd0922345d0",
          "message": "[otap-dataflow] Add internal telemetry to RetryProcessor (RFC-aligned) + tests (#1141)\n\n• Introduce retry.processor.metrics with RFC-style consumed/produced\nitem counters and component metrics.\n•  Wire metrics via PipelineContext and report on CollectTelemetry.\n• Instrument enqueue, ACK/NACK, retry send, queue-full,\nexceeded-retries, and cleanup paths.\n• Add unit tests for telemetry (collect, queue-full, exceeded-retries,\ncleanup).",
          "timestamp": "2025-09-17T22:06:07Z",
          "tree_id": "6cc4c6c29a4f9a6734445e6f015efde3188a03df",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/eaf81aabee7eaa80d4ffe78a59d9bfd0922345d0"
        },
        "date": 1758147256715,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 731500,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 21945000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 21945000,
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
            "value": 6.61,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 121.01,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 143.25,
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
            "value": 6.9,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 161.4,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 185.46,
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
          "id": "e889bc3484ea814259fc7dcba61a38c58d51fe8b",
          "message": "Encode OTAP -> OTLP proto bytes directly for Slice/Map AnyValues (#1147)\n\ncloses: #1121 \n\nStop going to the intermediate prost struct when decoding Map / Slice\nAnyValues",
          "timestamp": "2025-09-17T22:08:18Z",
          "tree_id": "5c8a93e20bed5ba709e55b976d62a93e48a47d08",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/e889bc3484ea814259fc7dcba61a38c58d51fe8b"
        },
        "date": 1758147406678,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 727500,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 21825000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 21825000,
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
            "value": 6.57,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 133.39,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 157.61,
            "unit": "MiB"
          },
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
            "value": 5.76,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 6.82,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 165.45,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 192.31,
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
          "id": "1ca74027095cbd46bda865b09d5cb80c94262574",
          "message": "[query-engine] Summary schema validation support in KQL parser (#1148)\n\n## Changes\n\n* Support validation of summary schema in KQL parser",
          "timestamp": "2025-09-18T16:41:36Z",
          "tree_id": "87af26c1b3ee52337d802bfbd60a2ca7229307b8",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/1ca74027095cbd46bda865b09d5cb80c94262574"
        },
        "date": 1758214666870,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 732000,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 21960000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 21960000,
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
            "value": 6.82,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 130.58,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 157.07,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 724833.3333333334,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 21745000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 21745000,
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
            "value": 6.79,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 157.57,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 181.45,
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
          "id": "e200625cdbd6d41b0c679d7339e9776c747f4cdf",
          "message": "[otap-df-otap] Fix indentation (#1153)\n\n## Changes\n- Fix indentation",
          "timestamp": "2025-09-18T20:54:50Z",
          "tree_id": "cae2f0e63f45891223067d2d97d4ca5a4ad19f84",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/e200625cdbd6d41b0c679d7339e9776c747f4cdf"
        },
        "date": 1758229376222,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 733000,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 21990000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 21990000,
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
            "value": 6.78,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 126.4,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 150.94,
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
            "value": 6.6,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 171.7,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 198.34,
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
          "id": "1c80b0cd77e7c7dd2480cdb4511d069ceaeda73c",
          "message": "[query-engine] Support passing attribute schema in OTLP bridge (#1151)\n\nnt",
          "timestamp": "2025-09-18T22:10:38Z",
          "tree_id": "61447da43f8605232e4f6341ffca4bcc4661c9df",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/1c80b0cd77e7c7dd2480cdb4511d069ceaeda73c"
        },
        "date": 1758233953307,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 729166.6666666666,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 21875000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 21875000,
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
            "value": 6.7,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 124.05,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 145.42,
            "unit": "MiB"
          },
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
            "value": 5.65,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 6.78,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 160.16,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 188.21,
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
          "id": "b1f9fd40581f8e306c9a1b63866fde5f3f0b1835",
          "message": "fix failing rust CI jobs (#1162)\n\nWe've seen som CI failures\nhttps://github.com/open-telemetry/otel-arrow/actions/runs/17867782439/job/50814170759?pr=1160\n\ntrying these changes to see if it fixes CI:\n- set msrv\n- update the msrv in .clippy.toml files",
          "timestamp": "2025-09-22T14:31:56Z",
          "tree_id": "9a801c4950e8d419ef20b6f15dd4dfc86635e7a6",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/b1f9fd40581f8e306c9a1b63866fde5f3f0b1835"
        },
        "date": 1758552017776,
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
            "value": 5.7,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 6.7,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 123.05,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 144.96,
            "unit": "MiB"
          },
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
            "value": 5.55,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 6.72,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 167.98,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 197.93,
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
          "id": "40e2d5618e0078b2fbe7911db21607606152ed8f",
          "message": "chore(deps): update dependency duckdb to v1.4.0 (#1163)\n\nComing soon: The Renovate bot (GitHub App) will be renamed to Mend. PRs\nfrom Renovate will soon appear from 'Mend'. Learn more\n[here](https://redirect.github.com/renovatebot/renovate/discussions/37842).\n\nThis PR contains the following updates:\n\n| Package | Change | Age | Confidence |\n|---|---|---|---|\n| [duckdb](https://redirect.github.com/duckdb/duckdb-python)\n([changelog](https://redirect.github.com/duckdb/duckdb/releases)) |\n`==1.3.2` -> `==1.4.0` |\n[![age](https://developer.mend.io/api/mc/badges/age/pypi/duckdb/1.4.0?slim=true)](https://docs.renovatebot.com/merge-confidence/)\n|\n[![confidence](https://developer.mend.io/api/mc/badges/confidence/pypi/duckdb/1.3.2/1.4.0?slim=true)](https://docs.renovatebot.com/merge-confidence/)\n|\n\n---\n\n### Release Notes\n\n<details>\n<summary>duckdb/duckdb-python (duckdb)</summary>\n\n###\n[`v1.4.0`](https://redirect.github.com/duckdb/duckdb-python/compare/v1.3.2...v1.4.0)\n\n[Compare\nSource](https://redirect.github.com/duckdb/duckdb-python/compare/v1.3.2...v1.4.0)\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - \"before 8am on Monday\" (UTC),\nAutomerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0MS45Ny4xMCIsInVwZGF0ZWRJblZlciI6IjQxLjk3LjEwIiwidGFyZ2V0QnJhbmNoIjoibWFpbiIsImxhYmVscyI6WyJkZXBlbmRlbmNpZXMiXX0=-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2025-09-22T14:48:41Z",
          "tree_id": "52e670fdec261fb492bce3c85e95a96312a56c68",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/40e2d5618e0078b2fbe7911db21607606152ed8f"
        },
        "date": 1758552998588,
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
            "value": 5.63,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 6.78,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 124.37,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 144.7,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 727500,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 21825000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 21825000,
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
            "value": 6.82,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 158.39,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 188.48,
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
          "id": "ee99b8ba8ac4da587d2e4d31f3132c60c2f90fe1",
          "message": "chore(deps): update dependency prometheus_client to v0.23.1 (#1165)\n\nComing soon: The Renovate bot (GitHub App) will be renamed to Mend. PRs\nfrom Renovate will soon appear from 'Mend'. Learn more\n[here](https://redirect.github.com/renovatebot/renovate/discussions/37842).\n\nThis PR contains the following updates:\n\n| Package | Change | Age | Confidence |\n|---|---|---|---|\n|\n[prometheus_client](https://redirect.github.com/prometheus/client_python)\n| `==0.22.1` -> `==0.23.1` |\n[![age](https://developer.mend.io/api/mc/badges/age/pypi/prometheus-client/0.23.1?slim=true)](https://docs.renovatebot.com/merge-confidence/)\n|\n[![confidence](https://developer.mend.io/api/mc/badges/confidence/pypi/prometheus-client/0.22.1/0.23.1?slim=true)](https://docs.renovatebot.com/merge-confidence/)\n|\n\n---\n\n### Release Notes\n\n<details>\n<summary>prometheus/client_python (prometheus_client)</summary>\n\n###\n[`v0.23.1`](https://redirect.github.com/prometheus/client_python/releases/tag/v0.23.1)\n\n[Compare\nSource](https://redirect.github.com/prometheus/client_python/compare/v0.23.0...v0.23.1)\n\n##### What's Changed\n\n- fix: use tuples instead of packaging Version by\n[@&#8203;efiop](https://redirect.github.com/efiop) in\n[#&#8203;1136](https://redirect.github.com/prometheus/client_python/pull/1136)\n\n##### New Contributors\n\n- [@&#8203;efiop](https://redirect.github.com/efiop) made their first\ncontribution in\n[#&#8203;1136](https://redirect.github.com/prometheus/client_python/pull/1136)\n\n**Full Changelog**:\n<https://github.com/prometheus/client_python/compare/v0.23.0...v0.23.1>\n\n###\n[`v0.23.0`](https://redirect.github.com/prometheus/client_python/releases/tag/v0.23.0)\n\n[Compare\nSource](https://redirect.github.com/prometheus/client_python/compare/v0.22.1...v0.23.0)\n\n##### What's Changed\n\n- UTF-8 Content Negotiation by\n[@&#8203;ywwg](https://redirect.github.com/ywwg) in\n[#&#8203;1102](https://redirect.github.com/prometheus/client_python/pull/1102)\n- Re include test data by\n[@&#8203;mgorny](https://redirect.github.com/mgorny) in\n[#&#8203;1113](https://redirect.github.com/prometheus/client_python/pull/1113)\n- Improve parser performance by\n[@&#8203;csmarchbanks](https://redirect.github.com/csmarchbanks) in\n[#&#8203;1117](https://redirect.github.com/prometheus/client_python/pull/1117)\n- Add support to `write_to_textfile` for custom tmpdir by\n[@&#8203;aadityadhruv](https://redirect.github.com/aadityadhruv) in\n[#&#8203;1115](https://redirect.github.com/prometheus/client_python/pull/1115)\n- OM text exposition for NH by\n[@&#8203;vesari](https://redirect.github.com/vesari) in\n[#&#8203;1087](https://redirect.github.com/prometheus/client_python/pull/1087)\n- Fix bug which caused metric publishing to not accept query string\nparameters in ASGI app by\n[@&#8203;hacksparr0w](https://redirect.github.com/hacksparr0w) in\n[#&#8203;1125](https://redirect.github.com/prometheus/client_python/pull/1125)\n- Emit native histograms only when OM 2.0.0 is requested by\n[@&#8203;vesari](https://redirect.github.com/vesari) in\n[#&#8203;1128](https://redirect.github.com/prometheus/client_python/pull/1128)\n- fix: remove space after comma in openmetrics exposition by\n[@&#8203;theSuess](https://redirect.github.com/theSuess) in\n[#&#8203;1132](https://redirect.github.com/prometheus/client_python/pull/1132)\n- Fix issue parsing double spaces after # HELP/# TYPE by\n[@&#8203;csmarchbanks](https://redirect.github.com/csmarchbanks) in\n[#&#8203;1134](https://redirect.github.com/prometheus/client_python/pull/1134)\n\n##### New Contributors\n\n- [@&#8203;mgorny](https://redirect.github.com/mgorny) made their first\ncontribution in\n[#&#8203;1113](https://redirect.github.com/prometheus/client_python/pull/1113)\n- [@&#8203;aadityadhruv](https://redirect.github.com/aadityadhruv) made\ntheir first contribution in\n[#&#8203;1115](https://redirect.github.com/prometheus/client_python/pull/1115)\n- [@&#8203;hacksparr0w](https://redirect.github.com/hacksparr0w) made\ntheir first contribution in\n[#&#8203;1125](https://redirect.github.com/prometheus/client_python/pull/1125)\n- [@&#8203;theSuess](https://redirect.github.com/theSuess) made their\nfirst contribution in\n[#&#8203;1132](https://redirect.github.com/prometheus/client_python/pull/1132)\n\n**Full Changelog**:\n<https://github.com/prometheus/client_python/compare/v0.22.1...v0.23.0>\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - \"before 8am on Monday\" (UTC),\nAutomerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0MS45Ny4xMCIsInVwZGF0ZWRJblZlciI6IjQxLjk3LjEwIiwidGFyZ2V0QnJhbmNoIjoibWFpbiIsImxhYmVscyI6WyJkZXBlbmRlbmNpZXMiXX0=-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2025-09-22T15:21:44Z",
          "tree_id": "38f0a4552fc2d5b8662db081637725d2995c9b58",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/ee99b8ba8ac4da587d2e4d31f3132c60c2f90fe1"
        },
        "date": 1758554987027,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 736333.3333333334,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 22090000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 22090000,
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
            "value": 6.63,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 116.87,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 137.1,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 727500,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 21825000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 21825000,
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
            "value": 164.54,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 188.82,
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
          "id": "bfbad6626e80331edfa1e3b00e4501006143aecf",
          "message": "[perf][otap-df-otap] Avoid unnecessary allocations in AttributeProcessor (#1168)\n\n## Changes\n- Avoid cloning the record batch in `apply_transform_with_stats` method\n- Avoid vec allocation for holding `ArrowPayloadType` s by precomputing\nthem for the different combinations\n\n---------\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2025-09-22T16:51:12Z",
          "tree_id": "9b5bd33465c4ee3d8682d3272fbcd88c8becf9ed",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/bfbad6626e80331edfa1e3b00e4501006143aecf"
        },
        "date": 1758560323673,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 728333.3333333334,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 21850000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 21850000,
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
            "value": 6.67,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 123.51,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 145.05,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 735166.6666666666,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 22055000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 22055000,
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
            "value": 6.71,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 165.65,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 197.74,
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
          "id": "dcd42076942b4eb1e738ee65b9ec19df12882eee",
          "message": "[otap-df-otap] Refactor AttributesProcessor (#1169)\n\n## Changes\n- Introduce a convenient `const fn noop()` to check if there are any\ntransforms to apply\n- Make `attrs_payloads` and `apply_transform_with_stats` methods of\n`AttributeProcessor`. This provides better enapsulation removing the\nneed to pass `&self.transform` and `&self.domains` as parameters",
          "timestamp": "2025-09-22T21:26:59Z",
          "tree_id": "06a1af6931e4edc159b96ef6201f69afe9e69508",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/dcd42076942b4eb1e738ee65b9ec19df12882eee"
        },
        "date": 1758576979346,
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
            "value": 5.52,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 6.56,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 126.62,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 146.8,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 744333.3333333334,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 22330000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 22330000,
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
            "value": 170.08,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 201.47,
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
          "id": "72628f358a7a69e1b51bfd2bbd61cf9fa29009fb",
          "message": "[perf][otap-df-otap] Avoid redundant allocations for validating AttributeTransform config (#1172)\n\n## Changes\n- The transforms for `AttributesProcessor` are not changed after\ninitialization\n- Currently we are `AttributesTransform::validate()` in the hot-path.\nThis method allocates a `BTreeSet` when called\n- In this PR, I have updated the code to perform validation of\n`AttributesTransform` just one in the `new` method",
          "timestamp": "2025-09-22T22:57:26Z",
          "tree_id": "8e7ce8c8a5baf6c65268f5411e46ad305e05d92a",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/72628f358a7a69e1b51bfd2bbd61cf9fa29009fb"
        },
        "date": 1758582303489,
        "tool": "customSmallerIsBetter",
        "benches": [
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
            "value": 5.54,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 6.73,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 134.87,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 157.22,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 739500,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 22185000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 22185000,
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
            "value": 171.55,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 212.04,
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
          "id": "b8ee8073ebf5862fcea798f889465c038982cd9d",
          "message": "[query-engine] KQL Pratt parsing (#1174)\n\n## Changes\n\n* Use `PrattParser` in KQL parser to resolve operator precedence\n\n## Details\n\nPest doesn't support left-hand recursion. To workaround that issue up to\nthis point we have forced parenthesis in recursive rules:\n\n`source | extend a = (1 + 1)`\n`source | extend is_something = (key == 'name')`\n\nNow you can do:\n\n`source | extend a = 1 + 1`\n`source | extend is_something = key1 == 'something'`\n\nWhich is more natural and allowed in KQL.\n\nBased on a very nice\n[PoC](https://github.com/gouslu/otel-arrow/blob/gouslu/recursion/rust/experimental/query_engine/kql-parser/src/pratt_arithmetic.rs)\ndone by @gouslu.",
          "timestamp": "2025-09-23T20:52:45Z",
          "tree_id": "3eddb891dd4f1a62dd321d3ff31e6d07e7809600",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/b8ee8073ebf5862fcea798f889465c038982cd9d"
        },
        "date": 1758661640055,
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
            "value": 127.02,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 153.69,
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
            "value": 5.59,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 6.75,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 162.72,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 186.6,
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
          "id": "35d23213e095b1342d79231f17fc0f55db1a9e1e",
          "message": "[query-engine] Improve formatting of diagnostics when displayed (#1175)\n\n## Changes\n\n* Attempting to improve the readability of recordset engine diagnostics\nwhen displayed with the query\n\n## Details\n\n### Before\n\n```\nln   1: source | where (key1 == int(null) or key1 != int(null) or key1 == int(null)) and (name == int(null))\n                                                                                                  [Verbose] StaticScalar(Null): Evaluated as: 'null'\n                                                                                               [Verbose] LogicalExpression(EqualTo): Evaluated as: 'true'\n                                                                                          [Verbose] StaticScalar(String): Evaluated as: 'Attributes'\n                                                                                          [Verbose] StaticScalar(String): Resolved '{\"key1\":null,\"name\":null}' value for key 'Attributes' specified in accessor expression\n                                                                                          [Verbose] StaticScalar(String): Evaluated as: 'name'\n                                                                                          [Verbose] StaticScalar(String): Resolved 'null' value for key 'name' specified in accessor expression\n                                                                                          [Verbose] ScalarExpression(Source): Evaluated as: 'null'\n                                                                                     [Verbose] LogicalExpression(And): Evaluated as: 'true'\n                                                               [Verbose] LogicalExpression(Or): Evaluated as: 'true'\n                                          [Verbose] LogicalExpression(Or): Evaluated as: 'true'\n                                [Verbose] StaticScalar(Null): Evaluated as: 'null'\n                             [Verbose] LogicalExpression(EqualTo): Evaluated as: 'true'\n                        [Verbose] StaticScalar(String): Evaluated as: 'Attributes'\n                        [Verbose] StaticScalar(String): Resolved '{\"key1\":null,\"name\":null}' value for key 'Attributes' specified in accessor expression\n                        [Verbose] StaticScalar(String): Evaluated as: 'key1'\n                        [Verbose] StaticScalar(String): Resolved 'null' value for key 'key1' specified in accessor expression\n                        [Verbose] ScalarExpression(Source): Evaluated as: 'null'\n                 [Verbose] LogicalExpression(Not): Evaluated as: 'false'\n                 [Verbose] DiscardDataExpression: Record included\n```\n\n### After\n\n```\nln   1: source | where (key1 == int(null) or key1 != int(null) or key1 == int(null)) and (name == int(null))\n                 |      |    |  |         |                    |                     |    |    |  | [Verbose] StaticScalar(Null): Evaluated as: 'null'\n                 |      |    |  |         |                    |                     |    |    | [Verbose] LogicalExpression(EqualTo): Evaluated as: 'true'\n                 |      |    |  |         |                    |                     |    | [Verbose] StaticScalar(String): Evaluated as: 'Attributes'\n                 |      |    |  |         |                    |                     |    | [Verbose] StaticScalar(String): Resolved '{\"key1\":null,\"name\":null}' value for key 'Attributes' specified in accessor expression\n                 |      |    |  |         |                    |                     |    | [Verbose] StaticScalar(String): Evaluated as: 'name'\n                 |      |    |  |         |                    |                     |    | [Verbose] StaticScalar(String): Resolved 'null' value for key 'name' specified in accessor expression\n                 |      |    |  |         |                    |                     |    | [Verbose] ScalarExpression(Source): Evaluated as: 'null'\n                 |      |    |  |         |                    |                     | [Verbose] LogicalExpression(And): Evaluated as: 'true'\n                 |      |    |  |         |                    | [Verbose] LogicalExpression(Or): Evaluated as: 'true'\n                 |      |    |  |         | [Verbose] LogicalExpression(Or): Evaluated as: 'true'\n                 |      |    |  | [Verbose] StaticScalar(Null): Evaluated as: 'null'\n                 |      |    | [Verbose] LogicalExpression(EqualTo): Evaluated as: 'true'\n                 |      | [Verbose] StaticScalar(String): Evaluated as: 'Attributes'\n                 |      | [Verbose] StaticScalar(String): Resolved '{\"key1\":null,\"name\":null}' value for key 'Attributes' specified in accessor expression\n                 |      | [Verbose] StaticScalar(String): Evaluated as: 'key1'\n                 |      | [Verbose] StaticScalar(String): Resolved 'null' value for key 'key1' specified in accessor expression\n                 |      | [Verbose] ScalarExpression(Source): Evaluated as: 'null'\n                 | [Verbose] LogicalExpression(Not): Evaluated as: 'false'\n                 | [Verbose] DiscardDataExpression: Record included\n```",
          "timestamp": "2025-09-23T22:07:28Z",
          "tree_id": "3b3d696e42e8624195977f9014e35d41b1559541",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/35d23213e095b1342d79231f17fc0f55db1a9e1e"
        },
        "date": 1758667249425,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 723833.3333333334,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 21715000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 21715000,
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
            "value": 6.59,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 122.15,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 141.47,
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
            "value": 5.75,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 6.66,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 170.26,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 204.18,
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
          "id": "72f9be6efe77d6ab36cf0dbb11e8caf07468dc90",
          "message": "[perf][otap-df-otap] Avoid HashSet lookups in AttributesProcessor (#1173)\n\n## Changes\n- Precompute domain checks into `bool` variables to avoid `HashSet`\nlookup in the hot-path\n- This also allows to mark `attrs_payloads` method as `const`",
          "timestamp": "2025-09-24T20:26:25Z",
          "tree_id": "c0b43d25104ee7b5092ed724e3a2db07bea3962c",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/72f9be6efe77d6ab36cf0dbb11e8caf07468dc90"
        },
        "date": 1758746054728,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 732000,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 21960000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 21960000,
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
            "value": 125.05,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 147.39,
            "unit": "MiB"
          },
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
            "value": 5.7,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 6.77,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 160.21,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 189.69,
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
          "id": "3691b46508e67aba4b52e136974e19fc70f76a8d",
          "message": "[query-engine] Update Conditional formatting when displayed (#1179)\n\nI had intended for this to be part of #1178 but forgot to push. Sorry!",
          "timestamp": "2025-09-24T21:03:28Z",
          "tree_id": "bf9daa35eec2c3819b4ca7667241fdcca2f270a6",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/3691b46508e67aba4b52e136974e19fc70f76a8d"
        },
        "date": 1758748855155,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 723333.3333333334,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 21700000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 21700000,
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
            "value": 6.55,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 125.04,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 146.18,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 735666.6666666666,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 22070000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 22070000,
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
            "value": 7.16,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 163.57,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 187.42,
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
          "id": "7398745790e48fe41071bfece21194be4b55986d",
          "message": "Upgrade collector dependencies to 0.136.0/1.42.0 (#1176)",
          "timestamp": "2025-09-25T18:53:31Z",
          "tree_id": "abb897f839358fff1296c3b6f76eed0b96654679",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/7398745790e48fe41071bfece21194be4b55986d"
        },
        "date": 1758827825524,
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
            "value": 6.54,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 121.44,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 138.89,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 734833.3333333334,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 22045000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 22045000,
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
            "value": 6.92,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 168.27,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 204.71,
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
          "id": "507720481af23f8ea761dbf2dfc77516dd70c0e4",
          "message": "[query-engine] Implement fmt_with_indent for more expressions (#1182)\n\nRelates to #1178\n\n## Changes\n\n* Implements `fmt_with_indent` for more expressions. Note: I had CoPilot\ndo the initial work but I tweaked and cleaned everything up manually.\n\n## Details\n\nExamples:\n\n```\nPipeline\n├── Query: \"\"\n├── Constants: []\n├── Initializations: []\n└── Expressions:\n    └── Remove\n        └── Target(Mutable): Variable\n                             ├── Name: \"var\"\n                             └── Accessor:\n                                 └── String: \"field\"\n```\n\n```\nPipeline\n├── Query: \"\"\n├── Constants: []\n├── Initializations: []\n└── Expressions:\n    └── RemoveMapKeys(Retain)\n        ├── Target(Mutable): Source\n        │                    └── Accessor:\n        │                        └── String: \"Attributes\"\n        ├── Keys[0](Scalar): Variable\n        │                    ├── Name: \"var1\"\n        │                    └── Accessor: None\n        └── Keys[1](Scalar): Variable\n                             ├── Name: \"var2\"\n                             └── Accessor: None\n```\n\n```\nPipeline\n├── Query: \"\\nsource\\n | project-rename name = key1\\n | project-rename name = key1, key1 = source.Attributes[name]\\n | project-away key1['field'], name, *Time\\n \"\n├── Constants: []\n├── Initializations: []\n└── Expressions:\n    ├── Move\n    │   ├── Source(Mutable): Source\n    │   │                    └── Accessor:\n    │   │                        ├── String: \"Attributes\"\n    │   │                        └── String: \"key1\"\n    │   └── Destination(Mutable): Source\n    │                             └── Accessor:\n    │                                 ├── String: \"Attributes\"\n    │                                 └── String: \"name\"\n    ├── RenameMapKeys\n    │   ├── Target(Mutable): Source\n    │   │                    └── Accessor: None\n    │   ├── Keys[0]:\n    │   │   ├── Source(Accessor):\n    │   │   │   ├── String: \"Attributes\"\n    │   │   │   └── String: \"key1\"\n    │   │   └── Destination(Accessor):\n    │   │       ├── String: \"Attributes\"\n    │   │       └── String: \"name\"\n    │   └── Keys[1]:\n    │       ├── Source(Accessor):\n    │       │   ├── String: \"Attributes\"\n    │       │   └── Source\n    │       │       └── Accessor:\n    │       │           ├── String: \"Attributes\"\n    │       │           └── String: \"name\"\n    │       └── Destination(Accessor):\n    │           ├── String: \"Attributes\"\n    │           └── String: \"key1\"\n    └── ReduceMap(Remove)\n        ├── Target(Mutable): Source\n        │                    └── Accessor: None\n        ├── Selectors[0](KeyOrPattern): String: \"ObservedTimestamp\"\n        ├── Selectors[1](KeyOrPattern): String: \"Timestamp\"\n        ├── Selectors[2](Accessor):\n        │   ├── String: \"Attributes\"\n        │   ├── String: \"key1\"\n        │   └── String: \"field\"\n        ├── Selectors[3](Accessor):\n        │   ├── String: \"Attributes\"\n        │   └── Regex: \"^.*Time\"\n        └── Selectors[4](Accessor):\n            ├── String: \"Attributes\"\n            └── String: \"name\"\n```\n\n```\nPipeline\n├── Query: \"\\nsource\\n | extend\\n    c1 = coalesce(key1, name, 'n/a'),\\n    c2 = case(name == int(null), name, key1 == int(null), 'b', key1),\\n    l = strlen(name),\\n    s1 = substring(name, 5),\\n    s2 = substring(name, key1, key1)\\n \"\n├── Constants: []\n├── Initializations: []\n└── Expressions:\n    ├── Set\n    │   ├── Source(Scalar): Coalesce\n    │   │                   ├── Expressions[0](Scalar): Source\n    │   │                   │                           └── Accessor:\n    │   │                   │                               ├── String: \"Attributes\"\n    │   │                   │                               └── String: \"key1\"\n    │   │                   ├── Expressions[1](Scalar): Source\n    │   │                   │                           └── Accessor:\n    │   │                   │                               ├── String: \"Attributes\"\n    │   │                   │                               └── String: \"name\"\n    │   │                   └── Expressions[2](Scalar): String: \"n/a\"\n    │   └── Destination(Mutable): Source\n    │                             └── Accessor:\n    │                                 ├── String: \"Attributes\"\n    │                                 └── String: \"c1\"\n    ├── Set\n    │   ├── Source(Scalar): Case\n    │   │                   ├── When[0]\n    │   │                   │   ├── Condition(Logical): EqualTo\n    │   │                   │   │                       ├── Left(Scalar): Source\n    │   │                   │   │                       │                 └── Accessor:\n    │   │                   │   │                       │                     ├── String: \"Attributes\"\n    │   │                   │   │                       │                     └── String: \"name\"\n    │   │                   │   │                       └── Right(Scalar): Null\n    │   │                   │   └── Expression(Scalar): Source\n    │   │                   │                           └── Accessor:\n    │   │                   │                               ├── String: \"Attributes\"\n    │   │                   │                               └── String: \"name\"\n    │   │                   ├── When[1]\n    │   │                   │   ├── Condition(Logical): EqualTo\n    │   │                   │   │                       ├── Left(Scalar): Source\n    │   │                   │   │                       │                 └── Accessor:\n    │   │                   │   │                       │                     ├── String: \"Attributes\"\n    │   │                   │   │                       │                     └── String: \"key1\"\n    │   │                   │   │                       └── Right(Scalar): Null\n    │   │                   │   └── Expression(Scalar): String: \"b\"\n    │   │                   └── Else(Scalar): Source\n    │   │                                     └── Accessor:\n    │   │                                         ├── String: \"Attributes\"\n    │   │                                         └── String: \"key1\"\n    │   └── Destination(Mutable): Source\n    │                             └── Accessor:\n    │                                 ├── String: \"Attributes\"\n    │                                 └── String: \"c2\"\n    ├── Set\n    │   ├── Source(Scalar): Length(Scalar): Source\n    │   │                                   └── Accessor:\n    │   │                                       ├── String: \"Attributes\"\n    │   │                                       └── String: \"name\"\n    │   └── Destination(Mutable): Source\n    │                             └── Accessor:\n    │                                 ├── String: \"Attributes\"\n    │                                 └── String: \"l\"\n    ├── Set\n    │   ├── Source(Scalar): Slice\n    │   │                   ├── Source(Scalar): Source\n    │   │                   │                   └── Accessor:\n    │   │                   │                       ├── String: \"Attributes\"\n    │   │                   │                       └── String: \"name\"\n    │   │                   ├── StartInclusive(Scalar): Integer: 5\n    │   │                   └── EndExclusive: None\n    │   └── Destination(Mutable): Source\n    │                             └── Accessor:\n    │                                 ├── String: \"Attributes\"\n    │                                 └── String: \"s1\"\n    └── Set\n        ├── Source(Scalar): Slice\n        │                   ├── Source(Scalar): Source\n        │                   │                   └── Accessor:\n        │                   │                       ├── String: \"Attributes\"\n        │                   │                       └── String: \"name\"\n        │                   ├── StartInclusive(Scalar): Source\n        │                   │                           └── Accessor:\n        │                   │                               ├── String: \"Attributes\"\n        │                   │                               └── String: \"key1\"\n        │                   └── EndExclusive(Scalar): Source\n        │                                             └── Accessor:\n        │                                                 ├── String: \"Attributes\"\n        │                                                 └── String: \"key1\"\n        └── Destination(Mutable): Source\n                                  └── Accessor:\n                                      ├── String: \"Attributes\"\n                                      └── String: \"s2\"\n```",
          "timestamp": "2025-09-25T20:39:41Z",
          "tree_id": "3e0d2bbb63baa44464b2c33714df9d65fe7561ba",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/507720481af23f8ea761dbf2dfc77516dd70c0e4"
        },
        "date": 1758834695740,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 732166.6666666666,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 21965000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 21965000,
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
            "value": 6.67,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 128.64,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 152.32,
            "unit": "MiB"
          },
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
            "value": 5.73,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 6.69,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 160.12,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 187.27,
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
          "id": "40f7079e956539290bce48e0a802737743b5d91a",
          "message": "[query-engine] Fix divide by zero panics (#1183)\n\n## Changes\n\n* Fixes panics when doing division by `0` in `Math.Divide` and\n`Math.Bin`",
          "timestamp": "2025-09-26T15:06:32Z",
          "tree_id": "ad613a457dafaed2d8e9b53e6f3266387fde7d2c",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/40f7079e956539290bce48e0a802737743b5d91a"
        },
        "date": 1758899663383,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 736000,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 22080000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 22080000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 5.74,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 6.75,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 124.83,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 146.69,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 742166.6666666666,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 22265000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 22265000,
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
            "value": 6.75,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 162.52,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 190.05,
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
          "id": "ae15393c95adf7c29cf86f48caab1c0ca613197c",
          "message": "[query-engine] Implement fmt_with_indent for collection and conversion expressions (#1186)\n\nRelates to #1178\n\n## Details\n\nExample:\n\n```\nPipeline\n├── Query: \"\\nsource\\n | extend\\n    c1 = array_concat(parse_json('[]')),\\n    c2 = array_concat(name, key1),\\n    c3 = toint(key1)\\n \"\n├── Constants: []\n├── Initializations: []\n└── Expressions:\n    ├── Set\n    │   ├── Source(Scalar): Concat(Scalar): List\n    │   │                                   └── Values[0](Scalar): Array: []\n    │   └── Destination(Mutable): Source\n    │                             └── Accessor:\n    │                                 ├── String: \"Attributes\"\n    │                                 └── String: \"c1\"\n    ├── Set\n    │   ├── Source(Scalar): Concat(Scalar): List\n    │   │                                   ├── Values[0](Scalar): Source\n    │   │                                   │                      └── Accessor:\n    │   │                                   │                          ├── String: \"Attributes\"\n    │   │                                   │                          └── String: \"name\"\n    │   │                                   └── Values[1](Scalar): Source\n    │   │                                                          └── Accessor:\n    │   │                                                              ├── String: \"Attributes\"\n    │   │                                                              └── String: \"key1\"\n    │   └── Destination(Mutable): Source\n    │                             └── Accessor:\n    │                                 ├── String: \"Attributes\"\n    │                                 └── String: \"c2\"\n    └── Set\n        ├── Source(Scalar): ToInteger(Scalar): Source\n        │                                      └── Accessor:\n        │                                          ├── String: \"Attributes\"\n        │                                          └── String: \"key1\"\n        └── Destination(Mutable): Source\n                                  └── Accessor:\n                                      ├── String: \"Attributes\"\n                                      └── String: \"c3\"\n```",
          "timestamp": "2025-09-26T20:10:05Z",
          "tree_id": "83b7385d187a26cf2d32aa831b4758b6bf542778",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/ae15393c95adf7c29cf86f48caab1c0ca613197c"
        },
        "date": 1758917928242,
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
            "value": 5.63,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 6.63,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 125.64,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 148.36,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 739833.3333333334,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 22195000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 22195000,
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
            "value": 6.79,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 162.04,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 195.64,
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
          "id": "d838e0fcadf0571a36018bda695144d3b44ac65d",
          "message": "[query-engine] Remove top-level fields specified in attributes schema in otlp bridge (#1185)\n\n## Changes\n\n* Add special handling for top-level fields being specified as attribute\nschema in OTLP bridge",
          "timestamp": "2025-09-26T20:51:19Z",
          "tree_id": "f2c05d0a2666244bb981174f46127ac1a436a258",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/d838e0fcadf0571a36018bda695144d3b44ac65d"
        },
        "date": 1758920342920,
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
            "value": 5.68,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 6.76,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 129.74,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 156.2,
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
            "value": 5.69,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 7,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 158.04,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 186.67,
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
          "id": "21280b08c800cd857e9e5c3ace06f038358ba568",
          "message": "[perf] Avoid unnecessary vec allocation for merged transforms (#1180)\n\n## Changes\n- In the current implementation, we always allocate a `vec` when merging\ntransform ranges (even if there is no merge required for example if\nthere are either only replacement ranges or only delete ranges)\n- With this PR, we append the `KeyTransformRangeType` data from the\nget-go and store them in `KeyReplacementPlan` and `KeyDeletePlan`. This\nallows us to simply refer to these fully self-contained ranges in the\n`merge_transform_ranges` method if there is no merge required.",
          "timestamp": "2025-09-26T21:53:54Z",
          "tree_id": "aca724d353c102b4b3c5c82ea69f338718eb428d",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/21280b08c800cd857e9e5c3ace06f038358ba568"
        },
        "date": 1758924093306,
        "tool": "customSmallerIsBetter",
        "benches": [
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
            "value": 130.08,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 155,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 744166.6666666666,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 22325000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 22325000,
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
            "value": 6.67,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 163.35,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 191.52,
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
          "id": "8956a4f34f1ef246910c7d0f9d177ff0654cf2cf",
          "message": "chore(deps): update github workflow dependencies (#1192)\n\nComing soon: The Renovate bot (GitHub App) will be renamed to Mend. PRs\nfrom Renovate will soon appear from 'Mend'. Learn more\n[here](https://redirect.github.com/renovatebot/renovate/discussions/37842).\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n|---|---|---|---|\n|\n[github/codeql-action](https://redirect.github.com/github/codeql-action)\n| action | patch | `v3.30.3` -> `v3.30.5` |\n|\n[taiki-e/install-action](https://redirect.github.com/taiki-e/install-action)\n| action | minor | `v2.61.7` -> `v2.62.12` |\n\n---\n\n### Release Notes\n\n<details>\n<summary>github/codeql-action (github/codeql-action)</summary>\n\n###\n[`v3.30.5`](https://redirect.github.com/github/codeql-action/releases/tag/v3.30.5)\n\n[Compare\nSource](https://redirect.github.com/github/codeql-action/compare/v3.30.4...v3.30.5)\n\n##### CodeQL Action Changelog\n\nSee the [releases\npage](https://redirect.github.com/github/codeql-action/releases) for the\nrelevant changes to the CodeQL CLI and language packs.\n\n##### 3.30.5 - 26 Sep 2025\n\n- We fixed a bug that was introduced in `3.30.4` with `upload-sarif`\nwhich resulted in files without a `.sarif` extension not getting\nuploaded.\n[#&#8203;3160](https://redirect.github.com/github/codeql-action/pull/3160)\n\nSee the full\n[CHANGELOG.md](https://redirect.github.com/github/codeql-action/blob/v3.30.5/CHANGELOG.md)\nfor more information.\n\n###\n[`v3.30.4`](https://redirect.github.com/github/codeql-action/releases/tag/v3.30.4)\n\n[Compare\nSource](https://redirect.github.com/github/codeql-action/compare/v3.30.3...v3.30.4)\n\n##### CodeQL Action Changelog\n\nSee the [releases\npage](https://redirect.github.com/github/codeql-action/releases) for the\nrelevant changes to the CodeQL CLI and language packs.\n\n##### 3.30.4 - 25 Sep 2025\n\n- We have improved the CodeQL Action's ability to validate that the\nworkflow it is used in does not use different versions of the CodeQL\nAction for different workflow steps. Mixing different versions of the\nCodeQL Action in the same workflow is unsupported and can lead to\nunpredictable results. A warning will now be emitted from the\n`codeql-action/init` step if different versions of the CodeQL Action are\ndetected in the workflow file. Additionally, an error will now be thrown\nby the other CodeQL Action steps if they load a configuration file that\nwas generated by a different version of the `codeql-action/init` step.\n[#&#8203;3099](https://redirect.github.com/github/codeql-action/pull/3099)\nand\n[#&#8203;3100](https://redirect.github.com/github/codeql-action/pull/3100)\n- We added support for reducing the size of dependency caches for Java\nanalyses, which will reduce cache usage and speed up workflows. This\nwill be enabled automatically at a later time.\n[#&#8203;3107](https://redirect.github.com/github/codeql-action/pull/3107)\n- You can now run the latest CodeQL nightly bundle by passing `tools:\nnightly` to the `init` action. In general, the nightly bundle is\nunstable and we only recommend running it when directed by GitHub staff.\n[#&#8203;3130](https://redirect.github.com/github/codeql-action/pull/3130)\n- Update default CodeQL bundle version to 2.23.1.\n[#&#8203;3118](https://redirect.github.com/github/codeql-action/pull/3118)\n\nSee the full\n[CHANGELOG.md](https://redirect.github.com/github/codeql-action/blob/v3.30.4/CHANGELOG.md)\nfor more information.\n\n</details>\n\n<details>\n<summary>taiki-e/install-action (taiki-e/install-action)</summary>\n\n###\n[`v2.62.12`](https://redirect.github.com/taiki-e/install-action/blob/HEAD/CHANGELOG.md#100---2021-12-30)\n\n[Compare\nSource](https://redirect.github.com/taiki-e/install-action/compare/v2.62.11...v2.62.12)\n\nInitial release\n\n[Unreleased]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.12...HEAD\n\n[2.62.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.11...v2.62.12\n\n[2.62.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.10...v2.62.11\n\n[2.62.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.9...v2.62.10\n\n[2.62.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.8...v2.62.9\n\n[2.62.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.7...v2.62.8\n\n[2.62.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.6...v2.62.7\n\n[2.62.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.5...v2.62.6\n\n[2.62.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.4...v2.62.5\n\n[2.62.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.3...v2.62.4\n\n[2.62.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.2...v2.62.3\n\n[2.62.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.1...v2.62.2\n\n[2.62.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.0...v2.62.1\n\n[2.62.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.13...v2.62.0\n\n[2.61.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.12...v2.61.13\n\n[2.61.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.11...v2.61.12\n\n[2.61.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.10...v2.61.11\n\n[2.61.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.9...v2.61.10\n\n[2.61.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.8...v2.61.9\n\n[2.61.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.7...v2.61.8\n\n[2.61.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.6...v2.61.7\n\n[2.61.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.5...v2.61.6\n\n[2.61.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.4...v2.61.5\n\n[2.61.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.3...v2.61.4\n\n[2.61.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.2...v2.61.3\n\n[2.61.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.1...v2.61.2\n\n[2.61.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.0...v2.61.1\n\n[2.61.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.60.0...v2.61.0\n\n[2.60.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.59.1...v2.60.0\n\n[2.59.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.59.0...v2.59.1\n\n[2.59.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.33...v2.59.0\n\n[2.58.33]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.32...v2.58.33\n\n[2.58.32]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.31...v2.58.32\n\n[2.58.31]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.30...v2.58.31\n\n[2.58.30]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.29...v2.58.30\n\n[2.58.29]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.28...v2.58.29\n\n[2.58.28]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.27...v2.58.28\n\n[2.58.27]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.26...v2.58.27\n\n[2.58.26]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.25...v2.58.26\n\n[2.58.25]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.24...v2.58.25\n\n[2.58.24]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.23...v2.58.24\n\n[2.58.23]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.22...v2.58.23\n\n[2.58.22]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.21...v2.58.22\n\n[2.58.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.20...v2.58.21\n\n[2.58.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.19...v2.58.20\n\n[2.58.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.18...v2.58.19\n\n[2.58.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.17...v2.58.18\n\n[2.58.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.16...v2.58.17\n\n[2.58.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.15...v2.58.16\n\n[2.58.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.14...v2.58.15\n\n[2.58.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.13...v2.58.14\n\n[2.58.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.12...v2.58.13\n\n[2.58.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.11...v2.58.12\n\n[2.58.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.10...v2.58.11\n\n[2.58.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.9...v2.58.10\n\n[2.58.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.8...v2.58.9\n\n[2.58.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.7...v2.58.8\n\n[2.58.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.6...v2.58.7\n\n[2.58.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.5...v2.58.6\n\n[2.58.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.4...v2.58.5\n\n[2.58.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.3...v2.58.4\n\n[2.58.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.2...v2.58.3\n\n[2.58.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.1...v2.58.2\n\n[2.58.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.0...v2.58.1\n\n[2.58.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.8...v2.58.0\n\n[2.57.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.7...v2.57.8\n\n[2.57.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.6...v2.57.7\n\n[2.57.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.5...v2.57.6\n\n[2.57.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.4...v2.57.5\n\n[2.57.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.3...v2.57.4\n\n[2.57.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.2...v2.57.3\n\n[2.57.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.1...v2.57.2\n\n[2.57.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.0...v2.57.1\n\n[2.57.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.24...v2.57.0\n\n[2.56.24]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.23...v2.56.24\n\n[2.56.23]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.22...v2.56.23\n\n[2.56.22]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.21...v2.56.22\n\n[2.56.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.20...v2.56.21\n\n[2.56.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.19...v2.56.20\n\n[2.56.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.18...v2.56.19\n\n[2.56.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.17...v2.56.18\n\n[2.56.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.16...v2.56.17\n\n[2.56.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.15...v2.56.16\n\n[2.56.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.14...v2.56.15\n\n[2.56.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.13...v2.56.14\n\n[2.56.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.12...v2.56.13\n\n[2.56.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.11...v2.56.12\n\n[2.56.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.10...v2.56.11\n\n[2.56.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.9...v2.56.10\n\n[2.56.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.8...v2.56.9\n\n[2.56.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.7...v2.56.8\n\n[2.56.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.6...v2.56.7\n\n[2.56.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.5...v2.56.6\n\n[2.56.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.4...v2.56.5\n\n[2.56.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.3...v2.56.4\n\n[2.56.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.2...v2.56.3\n\n[2.56.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.1...v2.56.2\n\n[2.56.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.0...v2.56.1\n\n[2.56.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.55.4...v2.56.0\n\n[2.55.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.55.3...v2.55.4\n\n[2.55.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.55.2...v2.55.3\n\n[2.55.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.55.1...v2.55.2\n\n[2.55.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.55.0...v2.55.1\n\n[2.55.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.54.3...v2.55.0\n\n[2.54.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.54.2...v2.54.3\n\n[2.54.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.54.1...v2.54.2\n\n[2.54.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.54.0...v2.54.1\n\n[2.54.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.53.2...v2.54.0\n\n[2.53.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.53.1...v2.53.2\n\n[2.53.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.53.0...v2.53.1\n\n[2.53.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.8...v2.53.0\n\n[2.52.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.7...v2.52.8\n\n[2.52.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.6...v2.52.7\n\n[2.52.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.5...v2.52.6\n\n[2.52.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.4...v2.52.5\n\n[2.52.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.3...v2.52.4\n\n[2.52.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.2...v2.52.3\n\n[2.52.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.1...v2.52.2\n\n[2.52.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.0...v2.52.1\n\n[2.52.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.51.3...v2.52.0\n\n[2.51.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.51.2...v2.51.3\n\n[2.51.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.51.1...v2.51.2\n\n[2.51.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.51.0...v2.51.1\n\n[2.51.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.10...v2.51.0\n\n[2.50.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.9...v2.50.10\n\n[2.50.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.8...v2.50.9\n\n[2.50.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.7...v2.50.8\n\n[2.50.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.6...v2.50.7\n\n[2.50.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.5...v2.50.6\n\n[2.50.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.4...v2.50.5\n\n[2.50.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.3...v2.50.4\n\n[2.50.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.2...v2.50.3\n\n[2.50.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.1...v2.50.2\n\n[2.50.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.0...v2.50.1\n\n[2.50.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.50...v2.50.0\n\n[2.49.50]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.49...v2.49.50\n\n[2.49.49]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.48...v2.49.49\n\n[2.49.48]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.47...v2.49.48\n\n[2.49.47]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.46...v2.49.47\n\n[2.49.46]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.45...v2.49.46\n\n[2.49.45]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.44...v2.49.45\n\n[2.49.44]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.43...v2.49.44\n\n[2.49.43]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.42...v2.49.43\n\n[2.49.42]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.41...v2.49.42\n\n[2.49.41]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.40...v2.49.41\n\n[2.49.40]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.39...v2.49.40\n\n[2.49.39]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.38...v2.49.39\n\n[2.49.38]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.37...v2.49.38\n\n[2.49.37]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.36...v2.49.37\n\n[2.49.36]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.35...v2.49.36\n\n[2.49.35]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.34...v2.49.35\n\n[2.49.34]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.33...v2.49.34\n\n[2.49.33]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.32...v2.49.33\n\n[2.49.32]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.31...v2.49.32\n\n[2.49.31]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.30...v2.49.31\n\n[2.49.30]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.29...v2.49.30\n\n[2.49.29]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.28...v2.49.29\n\n[2.49.28]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.27...v2.49.28\n\n[2.49.27]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.26...v2.49.27\n\n[2.49.26]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.25...v2.49.26\n\n[2.49.25]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.24...v2.49.25\n\n[2.49.24]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.23...v2.49.24\n\n[2.49.23]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.22...v2.49.23\n\n[2.49.22]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.21...v2.49.22\n\n[2.49.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.20...v2.49.21\n\n[2.49.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.19...v2.49.20\n\n[2.49.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.18...v2.49.19\n\n[2.49.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.17...v2.49.18\n\n[2.49.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.16...v2.49.17\n\n[2.49.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.15...v2.49.16\n\n[2.49.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.14...v2.49.15\n\n[2.49.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.13...v2.49.14\n\n[2.49.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.12...v2.49.13\n\n[2.49.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.11...v2.49.12\n\n[2.49.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.10...v2.49.11\n\n[2.49.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.9...v2.49.10\n\n[2.49.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.8...v2.49.9\n\n[2.49.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.7...v2.49.8\n\n[2.49.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.6...v2.49.7\n\n[2.49.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.5...v2.49.6\n\n[2.49.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.4...v2.49.5\n\n[2.49.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.3...v2.49.4\n\n[2.49.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.2...v2.49.3\n\n[2.49.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.1...v2.49.2\n\n[2.49.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.0...v2.49.1\n\n[2.49.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.22...v2.49.0\n\n[2.48.22]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.21...v2.48.22\n\n[2.48.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.20...v2.48.21\n\n[2.48.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.19...v2.48.20\n\n[2.48.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.18...v2.48.19\n\n[2.48.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.17...v2.48.18\n\n[2.48.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.16...v2.48.17\n\n[2.48.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.15...v2.48.16\n\n[2.48.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.14...v2.48.15\n\n[2.48.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.13...v2.48.14\n\n[2.48.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.12...v2.48.13\n\n[2.48.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.11...v2.48.12\n\n[2.48.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.10...v2.48.11\n\n[2.48.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.9...v2.48.10\n\n[2.48.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.8...v2.48.9\n\n[2.48.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.7...v2.48.8\n\n[2.48.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.6...v2.48.7\n\n[2.48.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.5...v2.48.6\n\n[2.48.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.4...v2.48.5\n\n[2.48.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.3...v2.48.4\n\n[2.48.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.2...v2.48.3\n\n[2.48.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.1...v2.48.2\n\n[2.48.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.0...v2.48.1\n\n[2.48.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.32...v2.48.0\n\n[2.47.32]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.31...v2.47.32\n\n[2.47.31]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.30...v2.47.31\n\n[2.47.30]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.29...v2.47.30\n\n[2.47.29]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.28...v2.47.29\n\n[2.47.28]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.27...v2.47.28\n\n[2.47.27]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.26...v2.47.27\n\n[2.47.26]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.25...v2.47.26\n\n[2.47.25]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.24...v2.47.25\n\n[2.47.24]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.23...v2.47.24\n\n[2.47.23]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.22...v2.47.23\n\n[2.47.22]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.21...v2.47.22\n\n[2.47.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.20...v2.47.21\n\n[2.47.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.19...v2.47.20\n\n[2.47.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.18...v2.47.19\n\n[2.47.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.17...v2.47.18\n\n[2.47.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.16...v2.47.17\n\n[2.47.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.15...v2.47.16\n\n[2.47.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.14...v2.47.15\n\n[2.47.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.13...v2.47.14\n\n[2.47.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.12...v2.47.13\n\n[2.47.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.11...v2.47.12\n\n[2.47.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.10...v2.47.11\n\n[2.47.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.9...v2.47.10\n\n[2.47.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.8...v2.47.9\n\n[2.47.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.7...v2.47.8\n\n[2.47.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.6...v2.47.7\n\n[2.47.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.5...v2.47.6\n\n[2.47.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.4...v2.47.5\n\n[2.47.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.3...v2.47.4\n\n[2.47.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.2...v2.47.3\n\n[2.47.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.1...v2.47.2\n\n[2.47.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.0...v2.47.1\n\n[2.47.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.20...v2.47.0\n\n[2.46.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.19...v2.46.20\n\n[2.46.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.18...v2.46.19\n\n[2.46.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.17...v2.46.18\n\n[2.46.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.16...v2.46.17\n\n[2.46.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.15...v2.46.16\n\n[2.46.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.14...v2.46.15\n\n[2.46.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.13...v2.46.14\n\n[2.46.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.12...v2.46.13\n\n[2.46.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.11...v2.46.12\n\n[2.46.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.10...v2.46.11\n\n[2.46.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.9...v2.46.10\n\n[2.46.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.8...v2.46.9\n\n[2.46.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.7...v2.46.8\n\n[2.46.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.6...v2.46.7\n\n[2.46.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.5...v2.46.6\n\n[2.46.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.4...v2.46.5\n\n[2.46.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.3...v2.46.4\n\n[2.46.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.2...v2.46.3\n\n[2.46.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.1...v2.46.2\n\n[2.46.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.0...v2.46.1\n\n[2.46.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.15...v2.46.0\n\n[2.45.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.14...v2.45.15\n\n[2.45.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.13...v2.45.14\n\n[2.45.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.12...v2.45.13\n\n[2.45.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.11...v2.45.12\n\n[2.45.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.10...v2.45.11\n\n[2.45.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.9...v2.45.10\n\n[2.45.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.8...v2.45.9\n\n[2.45.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.7...v2.45.8\n\n[2.45.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.6...v2.45.7\n\n[2.45.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.5...v2.45.6\n\n[2.45.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.4...v2.45.5\n\n[2.45.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.3...v2.45.4\n\n[2.45.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.2...v2.45.3\n\n[2.45.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.1...v2.45.2\n\n[2.45.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.0...v2.45.1\n\n[2.45.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.72...v2.45.0\n\n[2.44.72]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.71...v2.44.72\n\n[2.44.71]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.70...v2.44.71\n\n[2.44.70]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.69...v2.44.70\n\n[2.44.69]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.68...v2.44.69\n\n[2.44.68]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.67...v2.44.68\n\n[2.44.67]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.66...v2.44.67\n\n[2.44.66]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.65...v2.44.66\n\n[2.44.65]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.64...v2.44.65\n\n[2.44.64]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.63...v2.44.64\n\n[2.44.63]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.62...v2.44.63\n\n[2.44.62]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.61...v2.44.62\n\n[2.44.61]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.60...v2.44.61\n\n[2.44.60]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.59...v2.44.60\n\n[2.44.59]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.58...v2.44.59\n\n[2.44.58]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.57...v2.44.58\n\n[2.44.57]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.56...v2.44.57\n\n[2.44.56]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.55...v2.44.56\n\n[2.44.55]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.54...v2.44.55\n\n[2.44.54]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.53...v2.44.54\n\n[2.44.53]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.52...v2.44.53\n\n[2.44.52]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.51...v2.44.52\n\n[2.44.51]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.50...v2.44.51\n\n[2.44.50]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.49...v2.44.50\n\n[2.44.49]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.48...v2.44.49\n\n[2.44.48]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.47...v2.44.48\n\n[2.44.47]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.46...v2.44.47\n\n[2.44.46]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.45...v2.44.46\n\n[2.44.45]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.44...v2.44.45\n\n[2.44.44]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.43...v2.44.44\n\n[2.44.43]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.42...v2.44.43\n\n[2.44.42]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.41...v2.44.42\n\n[2.44.41]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.40...v2.44.41\n\n[2.44.40]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.39...v2.44.40\n\n[2.44.39]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.38...v2.44.39\n\n[2.44.38]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.37...v2.44.38\n\n[2.44.37]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.36...v2.44.37\n\n[2.44.36]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.35...v2.44.36\n\n[2.44.35]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.34...v2.44.35\n\n[2.44.34]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.33...v2.44.34\n\n[2.44.33]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.32...v2.44.33\n\n[2.44.32]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.31...v2.44.32\n\n[2.44.31]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.30...v2.44.31\n\n[2.44.30]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.29...v2.44.30\n\n[2.44.29]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.28...v2.44.29\n\n[2.44.28]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.27...v2.44.28\n\n[2.44.27]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.26...v2.44.27\n\n[2.44.26]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.25...v2.44.26\n\n[2.44.25]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.24...v2.44.25\n\n[2.44.24]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.23...v2.44.24\n\n[2.44.23]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.22...v2.44.23\n\n[2.44.22]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.21...v2.44.22\n\n[2.44.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.20...v2.44.21\n\n[2.44.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.19...v2.44.20\n\n[2.44.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.18...v2.44.19\n\n[2.44.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.17...v2.44.18\n\n[2.44.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.16...v2.44.17\n\n[2.44.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.15...v2.44.16\n\n[2.44.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.14...v2.44.15\n\n[2.44.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.13...v2.44.14\n\n[2.44.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.12...v2.44.13\n\n[2.44.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.11...v2.44.12\n\n[2.44.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.10...v2.44.11\n\n[2.44.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.9...v2.44.10\n\n[2.44.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.8...v2.44.9\n\n[2.44.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.7...v2.44.8\n\n[2.44.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.6...v2.44.7\n\n[2.44.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.5...v2.44.6\n\n[2.44.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.4...v2.44.5\n\n[2.44.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.3...v2.44.4\n\n[2.44.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.2...v2.44.3\n\n[2.44.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.1...v2.44.2\n\n[2.44.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.0...v2.44.1\n\n[2.44.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.43.7...v2.44.0\n\n[2.43.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.43.6...v2.43.7\n\n[2.43.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.43.5...v2.43.6\n\n[2.43.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.43.4...v2.43.5\n\n[2.43.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.43.3...v2.43.4\n\n[2.43.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.43.2...v2.43.3\n\n[2.43.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.43.1...v2.43.2\n\n[2.43.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.43.0...v2.43.1\n\n[2.43.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.42...v2.43.0\n\n[2.42.42]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.41...v2.42.42\n\n[2.42.41]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.40...v2.42.41\n\n[2.42.40]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.39...v2.42.40\n\n[2.42.39]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.38...v2.42.39\n\n[2.42.38]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.37...v2.42.38\n\n[2.42.37]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.36...v2.42.37\n\n[2.42.36]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.35...v2.42.36\n\n[2.42.35]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.34...v2.42.35\n\n[2.42.34]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.33...v2.42.34\n\n[2.42.33]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.32...v2.42.33\n\n[2.42.32]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.31...v2.42.32\n\n[2.42.31]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.30...v2.42.31\n\n[2.42.30]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.29...v2.42.30\n\n[2.42.29]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.28...v2.42.29\n\n[2.42.28]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.27...v2.42.28\n\n[2.42.27]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.26...v2.42.27\n\n[2.42.26]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.25...v2.42.26\n\n[2.42.25]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.24...v2.42.25\n\n[2.42.24]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.23...v2.42.24\n\n[2.42.23]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.22...v2.42.23\n\n[2.42.22]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.21...v2.42.22\n\n[2.42.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.20...v2.42.21\n\n[2.42.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.19...v2.42.20\n\n[2.42.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.18...v2.42.19\n\n[2.42.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.17...v2.42.18\n\n[2.42.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.16...v2.42.17\n\n[2.42.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.15...v2.42.16\n\n[2.42.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.14...v2.42.15\n\n[2.42.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.13...v2.42.14\n\n[2.42.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.12...v2.42.13\n\n[2.42.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.11...v2.42.12\n\n[2.42.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.10...v2.42.11\n\n[2.42.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.9...v2.42.10\n\n[2.42.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.8...v2.42.9\n\n[2.42.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.7...v2.42.8\n\n[2.42.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.6...v2.42.7\n\n[2.42.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.5...v2.42.6\n\n[2.42.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.4...v2.42.5\n\n[2.42.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.3...v2.42.4\n\n[2.42.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.2...v2.42.3\n\n[2.42.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.1...v2.42.2\n\n[2.42.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.0...v2.42.1\n\n[2.42.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.18...v2.42.0\n\n[2.41.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.17...v2.41.18\n\n[2.41.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.16...v2.41.17\n\n[2.41.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.15...v2.41.16\n\n[2.41.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.14...v2.41.15\n\n[2.41.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.13...v2.41.14\n\n[2.41.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.12...v2.41.13\n\n[2.41.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.11...v2.41.12\n\n[2.41.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.10...v2.41.11\n\n[2.41.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.9...v2.41.10\n\n[2.41.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.8...v2.41.9\n\n[2.41.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.7...v2.41.8\n\n[2.41.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.6...v2.41.7\n\n[2.41.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.5...v2.41.6\n\n[2.41.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.4...v2.41.5\n\n[2.41.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.3...v2.41.4\n\n[2.41.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.2...v2.41.3\n\n[2.41.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.1...v2.41.2\n\n[2.41.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.0...v2.41.1\n\n[2.41.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.40.2...v2.41.0\n\n[2.40.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.40.1...v2.40.2\n\n[2.40.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.40.0...v2.40.1\n\n[2.40.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.39.2...v2.40.0\n\n[2.39.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.39.1...v2.39.2\n\n[2.39.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.39.0...v2.39.1\n\n[2.39.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.38.7...v2.39.0\n\n[2.38.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.38.6...v2.38.7\n\n[2.38.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.38.5...v2.38.6\n\n[2.38.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.38.4...v2.38.5\n\n[2.38.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.38.3...v2.38.4\n\n[2.38.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.38.2...v2.38.3\n\n[2.38.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.38.1...v2.38.2\n\n[2.38.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.38.0...v2.38.1\n\n[2.38.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.37.0...v2.38.0\n\n[2.37.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.36.0...v2.37.0\n\n[2.36.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.35.0...v2.36.0\n\n[2.35.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.34.3...v2.35.0\n\n[2.34.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.34.2...v2.34.3\n\n[2.34.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.34.1...v2.34.2\n\n[2.34.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.34.0...v2.34.1\n\n[2.34.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.36...v2.34.0\n\n[2.33.36]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.35...v2.33.36\n\n[2.33.35]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.34...v2.33.35\n\n[2.33.34]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.33...v2.33.34\n\n[2.33.33]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.32...v2.33.33\n\n[2.33.32]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.31...v2.33.32\n\n[2.33.31]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.30...v2.33.31\n\n[2.33.30]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.29...v2.33.30\n\n[2.33.29]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.28...v2.33.29\n\n[2.33.28]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.27...v2.33.28\n\n[2.33.27]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.26...v2.33.27\n\n[2.33.26]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.25...v2.33.26\n\n[2.33.25]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.24...v2.33.25\n\n[2.33.24]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.23...v2.33.24\n\n[2.33.23]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.22...v2.33.23\n\n[2.33.22]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.21...v2.33.22\n\n[2.33.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.20...v2.33.21\n\n[2.33.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.19...v2.33.20\n\n[2.33.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.18...v2.33.19\n\n[2.33.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.17...v2.33.18\n\n[2.33.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.16...v2.33.17\n\n[2.33.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.15...v2.33.16\n\n[2.33.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.14...v2.33.15\n\n[2.33.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.13...v2.33.14\n\n[2.33.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.12...v2.33.13\n\n[2.33.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.11...v2.33.12\n\n[2.33.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.10...v2.33.11\n\n[2.33.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.9...v2.33.10\n\n[2.33.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.8...v2.33.9\n\n[2.33.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.7...v2.33.8\n\n[2.33.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.6...v2.33.7\n\n[2.33.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.5...v2.33.6\n\n[2.33.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.4...v2.33.5\n\n[2.33.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.3...v2.33.4\n\n[2.33.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.2...v2.33.3\n\n[2.33.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.1...v2.33.2\n\n[2.33.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.0...v2.33.1\n\n[2.33.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.20...v2.33.0\n\n[2.32.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.19...v2.32.20\n\n[2.32.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.18...v2.32.19\n\n[2.32.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.17...v2.32.18\n\n[2.32.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.16...v2.32.17\n\n[2.32.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.15...v2.32.16\n\n[2.32.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.14...v2.32.15\n\n[2.32.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.13...v2.32.14\n\n[2.32.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.12...v2.32.13\n\n[2.32.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.11...v2.32.12\n\n[2.32.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.10...v2.32.11\n\n[2.32.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.9...v2.32.10\n\n[2.32.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.8...v2.32.9\n\n[2.32.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.7...v2.32.8\n\n[2.32.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.6...v2.32.7\n\n[2.32.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.5...v2.32.6\n\n[2.32.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.4...v2.32.5\n\n[2.32.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.3...v2.32.4\n\n[2.32.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.2...v2.32.3\n\n[2.32.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.1...v2.32.2\n\n[2.32.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.0...v2.32.1\n\n[2.32.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.31.3...v2.32.0\n\n[2.31.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.31.2...v2.31.3\n\n[2.31.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.31.1...v2.31.2\n\n[2.31.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.31.0...v2.31.1\n\n[2.31.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.30.0...v2.31.0\n\n[2.30.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.29.8...v2.30.0\n\n[2.29.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.29.7...v2.29.8\n\n[2.29.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.29.6...v2.29.7\n\n[2.29.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.29.5...v2.29.6\n\n[2.29.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.29.4...v2.29.5\n\n[2.29.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.29.3...v2.29.4\n\n[2.29.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.29.2...v2.29.3\n\n[2.29.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.29.1...v2.29.2\n\n[2.29.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.29.0...v2.29.1\n\n[2.29.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.16...v2.29.0\n\n[2.28.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.15...v2.28.16\n\n[2.28.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.14...v2.28.15\n\n[2.28.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.13...v2.28.14\n\n[2.28.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.12...v2.28.13\n\n[2.28.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.11...v2.28.12\n\n[2.28.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.10...v2.28.11\n\n[2.28.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.9...v2.28.10\n\n[2.28.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.8...v2.28.9\n\n[2.28.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.7...v2.28.8\n\n[2.28.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.6...v2.28.7\n\n[2.28.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.5...v2.28.6\n\n[2.28.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.4...v2.28.5\n\n[2.28.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.3...v2.28.4\n\n[2.28.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.2...v2.28.3\n\n[2.28.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.1...v2.28.2\n\n[2.28.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.0...v2.28.1\n\n[2.28.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.15...v2.28.0\n\n[2.27.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.14...v2.27.15\n\n[2.27.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.13...v2.27.14\n\n[2.27.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.12...v2.27.13\n\n[2.27.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.11...v2.27.12\n\n[2.27.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.10...v2.27.11\n\n[2.27.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.9...v2.27.10\n\n[2.27.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.8...v2.27.9\n\n[2.27.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.7...v2.27.8\n\n[2.27.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.6...v2.27.7\n\n[2.27.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.5...v2.27.6\n\n[2.27.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.4...v2.27.5\n\n[2.27.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.3...v2.27.4\n\n[2.27.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.2...v2.27.3\n\n[2.27.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.1...v2.27.2\n\n[2.27.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.0...v2.27.1\n\n[2.27.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.20...v2.27.0\n\n[2.26.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.19...v2.26.20\n\n[2.26.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.18...v2.26.19\n\n[2.26.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.17...v2.26.18\n\n[2.26.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.16...v2.26.17\n\n[2.26.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.15...v2.26.16\n\n[2.26.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.14...v2.26.15\n\n[2.26.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.13...v2.26.14\n\n[2.26.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.12...v2.26.13\n\n[2.26.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.11...v2.26.12\n\n[2.26.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.10...v2.26.11\n\n[2.26.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.9...v2.26.10\n\n[2.26.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.8...v2.26.9\n\n[2.26.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.7...v2.26.8\n\n[2.26.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.6...v2.26.7\n\n[2.26.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.5...v2.26.6\n\n[2.26.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.4...v2.26.5\n\n[2.26.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.3...v2.26.4\n\n[2.26.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.2...v2.26.3\n\n[2.26.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.1...v2.26.2\n\n[2.26.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.0...v2.26.1\n\n[2.26.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.25.11...v2.26.0\n\n[2.25.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.25.10...v2.25.11\n\n[2.25.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.25.9...v2.25.10\n\n[2.25.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.25.8...v2.25.9\n\n[2.25.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.25.7...v2.25.8\n\n[2.25.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.25.6...v2.25.7\n\n[2.25.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.25.5...v2.25.6\n\n[2.25.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.25.4...v2.25.5\n\n[2.25.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.25.3...v2.25.4\n\n[2.25.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.25.2...v2.25.3\n\n[2.25.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.25.1...v2.25.2\n\n[2.25.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.25.0...v2.25.1\n\n[2.25.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.24.4...v2.25.0\n\n[2.24.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.24.3...v2.24.4\n\n[2.24.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.24.2...v2.24.3\n\n[2.24.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.24.1...v2.24.2\n\n[2.24.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.24.0...v2.24.1\n\n[2.24.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.23.9...v2.24.0\n\n[2.23.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.23.8...v2.23.9\n\n[2.23.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.23.7...v2.23.8\n\n[2.23.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.23.6...v2.23.7\n\n[2.23.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.23.5...v2.23.6\n\n[2.23.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.23.4...v2.23.5\n\n[2.23.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.23.3...v2.23.4\n\n[2.23.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.23.2...v2.23.3\n\n[2.23.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.23.1...v2.23.2\n\n[2.23.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.23.0...v2.23.1\n\n[2.23.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.22.10...v2.23.0\n\n[2.22.10]: https://redirect.github.com/taiki-e/install-actio\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - \"before 8am on Monday\" (UTC),\nAutomerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n👻 **Immortal**: This PR will be recreated if closed unmerged. Get\n[config\nhelp](https://redirect.github.com/renovatebot/renovate/discussions) if\nthat's undesired.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0MS4xMzEuOSIsInVwZGF0ZWRJblZlciI6IjQxLjEzMS45IiwidGFyZ2V0QnJhbmNoIjoibWFpbiIsImxhYmVscyI6WyJkZXBlbmRlbmNpZXMiXX0=-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2025-09-29T14:56:27Z",
          "tree_id": "2615d2065a933a11c174333e9b916a58f1bd05f0",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/8956a4f34f1ef246910c7d0f9d177ff0654cf2cf"
        },
        "date": 1759158691748,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 728500,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 21855000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 21855000,
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
            "value": 6.57,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 118.16,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 138.25,
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
            "value": 5.53,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 6.59,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 158.13,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 184.48,
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
          "id": "207bf3dd8d59292200e6a43c273dba23090a6f77",
          "message": "[otap-df-otap] Fix CEF version parsing (#1191)\n\n## Changes\n- Fix CEF version parsing to comply with the CEF\n[spec](https://www.microfocus.com/documentation/arcsight/arcsight-smartconnectors-8.3/cef-implementation-standard/Content/CEF/Chapter%201%20What%20is%20CEF.htm).",
          "timestamp": "2025-09-29T18:23:36Z",
          "tree_id": "87f2c5018df27b92651b761568c943f5cc56af28",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/207bf3dd8d59292200e6a43c273dba23090a6f77"
        },
        "date": 1759170951514,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 733166.6666666666,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 21995000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 21995000,
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
            "value": 6.55,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 125.3,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 146.95,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 728166.6666666666,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 21845000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 21845000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-avg",
            "value": 5.56,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 6.79,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 157.48,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 182.23,
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
          "id": "e07dd861fae55013484a6e3cdb11455fb588461a",
          "message": "[query-engine] Implement fmt_with_indent for ParseScalarExpression (#1198)\n\nRelates to #1178\n\n## Details\n\nExample:\n\n```\nPipeline\n├── Query: \"\\nsource\\n | extend\\n    c1 = parse_json(json_attr),\\n    c2 = parse_regex(regex_attr),\\n    c3 = parse_regex(regex_attr, regex_options_attr)\\n \"\n├── Constants: []\n├── Initializations: []\n└── Expressions:\n    ├── Set\n    │   ├── Source(Scalar): ParseJson(Scalar): Source\n    │   │                                      └── Accessor:\n    │   │                                          ├── String: \"Attributes\"\n    │   │                                          └── String: \"json_attr\"\n    │   └── Destination(Mutable): Source\n    │                             └── Accessor:\n    │                                 ├── String: \"Attributes\"\n    │                                 └── String: \"c1\"\n    ├── Set\n    │   ├── Source(Scalar): ParseRegex\n    │   │                   ├── Pattern(Scalar): Source\n    │   │                   │                    └── Accessor:\n    │   │                   │                        ├── String: \"Attributes\"\n    │   │                   │                        └── String: \"regex_attr\"\n    │   │                   └── Options: None\n    │   └── Destination(Mutable): Source\n    │                             └── Accessor:\n    │                                 ├── String: \"Attributes\"\n    │                                 └── String: \"c2\"\n    └── Set\n        ├── Source(Scalar): ParseRegex\n        │                   ├── Pattern(Scalar): Source\n        │                   │                    └── Accessor:\n        │                   │                        ├── String: \"Attributes\"\n        │                   │                        └── String: \"regex_attr\"\n        │                   └── Options(Scalar): Source\n        │                                        └── Accessor:\n        │                                            ├── String: \"Attributes\"\n        │                                            └── String: \"regex_options_attr\"\n        └── Destination(Mutable): Source\n                                  └── Accessor:\n                                      ├── String: \"Attributes\"\n                                      └── String: \"c3\"\n```",
          "timestamp": "2025-09-29T22:28:38Z",
          "tree_id": "e61d6e030e4a2138a7ba5488edb31bec7c8eeea0",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/e07dd861fae55013484a6e3cdb11455fb588461a"
        },
        "date": 1759185379985,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 733000,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 21990000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 21990000,
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
            "value": 6.73,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 121.2,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 138.18,
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
            "value": 5.65,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 6.85,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 159.41,
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
          "id": "1ff8b77786b4c8164f93b8b30d62a7f4477bb41c",
          "message": "Add support for KQL not() (#1188)\n\nNoticed we have the `LogicalExpression` support for `not()` (used in\nconjunction with `where`) but never added this standalone in the KQL\ngrammar",
          "timestamp": "2025-09-30T15:30:50Z",
          "tree_id": "820652f52fd45f905acbe1ace7bcfa534f321629",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/1ff8b77786b4c8164f93b8b30d62a7f4477bb41c"
        },
        "date": 1759246732766,
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
            "value": 5.6,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 6.81,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 130.23,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 150.25,
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
            "value": 5.63,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 6.68,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 160.57,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 192.34,
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
          "distinct": false,
          "id": "a1dd1620b6ec7e6fc837b059076a42e299077fdd",
          "message": "chore(release) Prepare Release v0.43.0 (#1202)\n\n## Release v0.43.0\n\nThis PR prepares the repository for release v0.43.0.\n\n### Changes included:\n- Updated CHANGELOG.md with release notes\n- Updated collector/otelarrowcol-build.yaml version to v0.43.0\n- Updated collector/cmd/otelarrowcol/main.go version to v0.43.0\n\n### Release Notes:\n- Upgrade to v0.136.0 / v1.39.0 of collector dependencies.\n[#1176](https://github.com/open-telemetry/otel-arrow/pull/1176)\n\n### Checklist:\n- [x] Verify CHANGELOG.md formatting and content\n- [x] Verify collector version update in\ncollector/otelarrowcol-build.yaml\n- [x] Verify collector main.go version update in\ncollector/cmd/otelarrowcol/main.go\n- [x] Confirm all tests pass\n- [x] Ready to merge and tag release\n\nAfter merging this PR, run the **Push Release** workflow to create git\ntags and publish the GitHub release.\n\nCo-authored-by: otelbot <197425009+otelbot@users.noreply.github.com>",
          "timestamp": "2025-09-30T17:13:36Z",
          "tree_id": "1fc0c3cfb398807fc8c44bf676dbd3ea9492972f",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/a1dd1620b6ec7e6fc837b059076a42e299077fdd"
        },
        "date": 1759252920515,
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
            "value": 5.56,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 6.83,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 129.58,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 151.87,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 731166.6666666666,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 21935000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 21935000,
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
            "value": 6.75,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 149.61,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 177.19,
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
          "id": "106a102213cf76bc5f5dd973b680d09752660d68",
          "message": "[otap-df-otap] Update Syslog parser for improved RFC 3164 compliance (#1200)\n\n## Changes\n- Make priority optional for RFC 3164 messages to allow the parser to\nhandle messages without PRI header correctly\n- Removed the redundant field `message`",
          "timestamp": "2025-09-30T21:36:39Z",
          "tree_id": "431a3d5b186785d0a02eeff5a31be5027804b51f",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/106a102213cf76bc5f5dd973b680d09752660d68"
        },
        "date": 1759271926901,
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
            "value": 5.72,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 6.89,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 133.86,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 157.25,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 731000,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 21930000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 21930000,
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
            "value": 6.87,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 153.36,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 180.69,
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
          "id": "7e580486ce9f307caa52ff97cd21a93b92c1a9ac",
          "message": "[PerfTest] temporarily remove arrow payload compression in benchmark suites (#1204)\n\nUntil the Arrow-rs patch mentioned in\nhttps://github.com/open-telemetry/otel-arrow/issues/1129 is merged and\nin-use here, we need to disable the payload compression in the perf test\nbenchmarks.",
          "timestamp": "2025-10-01T00:20:08Z",
          "tree_id": "9359775f29374a69089ac83b27fe14c923dbd56a",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/7e580486ce9f307caa52ff97cd21a93b92c1a9ac"
        },
        "date": 1759278483892,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 718666.6666666666,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 21560000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 21560000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 5.44,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 6.65,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 124.05,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 149.94,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 720500,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 21615000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 21615000,
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
            "value": 6.87,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 159.01,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 188.26,
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
          "id": "c5b4386d171d4558c09e3305f1c958df79faaa38",
          "message": "[query-engine] Implement fmt_with_indent for text scalar expressions (#1205)\n\nRelates to #1178\n\n## Details\n\nExamples:\n\n```\nPipeline\n├── Query: \"\\nsource\\n | extend\\n    c1 = extract(regex, capture, value),\\n    c2 = strcat_delim(a, b),\\n    c3 = replace_string(field, needle, replacement)\\n \"\n├── Constants: []\n├── Initializations: []\n└── Expressions:\n    ├── Set\n    │   ├── Source(Scalar): Capture\n    │   │                   ├── Haystack(Scalar): Source\n    │   │                   │                     └── Accessor:\n    │   │                   │                         ├── String: \"Attributes\"\n    │   │                   │                         └── String: \"value\"\n    │   │                   ├── Pattern(Scalar): Source\n    │   │                   │                    └── Accessor:\n    │   │                   │                        ├── String: \"Attributes\"\n    │   │                   │                        └── String: \"regex\"\n    │   │                   └── CaptureGroup(Scalar): Source\n    │   │                                             └── Accessor:\n    │   │                                                 ├── String: \"Attributes\"\n    │   │                                                 └── String: \"capture\"\n    │   └── Destination(Mutable): Source\n    │                             └── Accessor:\n    │                                 ├── String: \"Attributes\"\n    │                                 └── String: \"c1\"\n    ├── Set\n    │   ├── Source(Scalar): Join\n    │   │                   ├── Separator(Scalar): Source\n    │   │                   │                      └── Accessor:\n    │   │                   │                          ├── String: \"Attributes\"\n    │   │                   │                          └── String: \"a\"\n    │   │                   └── Values(Scalar): List\n    │   │                                       └── Values[0](Scalar): Source\n    │   │                                                              └── Accessor:\n    │   │                                                                  ├── String: \"Attributes\"\n    │   │                                                                  └── String: \"b\"\n    │   └── Destination(Mutable): Source\n    │                             └── Accessor:\n    │                                 ├── String: \"Attributes\"\n    │                                 └── String: \"c2\"\n    └── Set\n        ├── Source(Scalar): Replace\n        │                   ├── Haystack(Scalar): Source\n        │                   │                     └── Accessor:\n        │                   │                         ├── String: \"Attributes\"\n        │                   │                         └── String: \"field\"\n        │                   ├── Needle(Scalar): Source\n        │                   │                   └── Accessor:\n        │                   │                       ├── String: \"Attributes\"\n        │                   │                       └── String: \"needle\"\n        │                   ├── Replacement(Scalar): Source\n        │                   │                        └── Accessor:\n        │                   │                            ├── String: \"Attributes\"\n        │                   │                            └── String: \"replacement\"\n        │                   └── CaseInsensitive: false\n        └── Destination(Mutable): Source\n                                  └── Accessor:\n                                      ├── String: \"Attributes\"\n                                      └── String: \"c3\"\n```",
          "timestamp": "2025-10-01T15:00:34Z",
          "tree_id": "1f358048f77c655fee49b08dfb9468e579121b98",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/c5b4386d171d4558c09e3305f1c958df79faaa38"
        },
        "date": 1759331335921,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 731000,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 21930000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 21930000,
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
            "value": 125.21,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 143.06,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 730000,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 21900000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 21900000,
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
            "value": 6.68,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 168.44,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 198.07,
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
          "id": "7c3962983ae901beebe451aba8d959a2ebabf17f",
          "message": "[query-engine] Improve validation in KQL accessor expressions when source is referenced directly (#1210)\n\n## Changes\n\n* Improve the schema validation in KQL accessor expressions when\n`source` is referenced directly\n\n## Details\n\nWe have schema validation in the KQL parser. When schema is defined\nwe'll use it to resolve types and detect invalid usage when possible.\n\nCurrently we only do schema validation when `source` is referenced\nimplicitly. For example `extend c = SomeField`. What this PR does is\nextend that validation to also run when `source` is referenced\nexplicitly for example `extend c = source.SomeField`.",
          "timestamp": "2025-10-01T19:10:50Z",
          "tree_id": "a9d5158306079abcb1a30bf1bf1f2c1dbb824481",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/7c3962983ae901beebe451aba8d959a2ebabf17f"
        },
        "date": 1759347357774,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 720666.6666666666,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 21620000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 21620000,
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
            "value": 6.56,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 128.87,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 152.34,
            "unit": "MiB"
          },
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
            "value": 5.66,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 6.77,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 159.23,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 182.69,
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
          "id": "9085affca27a927e83ac44f23aa98ec775cf2219",
          "message": "[otap-df-otap] Fix signal type input in OTAP Exporter (#1215)\n\n## Changes\n- Use the argument `signal_type` instead of hardcoded `SignalType::Logs`",
          "timestamp": "2025-10-02T14:37:56Z",
          "tree_id": "9e4507613349c9c7cd2dd6dc122476a33714c972",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/9085affca27a927e83ac44f23aa98ec775cf2219"
        },
        "date": 1759423391833,
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
            "value": 5.62,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 6.7,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 120.45,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 139.99,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 733000,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 21990000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 21990000,
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
            "value": 6.76,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 160.1,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 187.78,
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
          "id": "55aaa48e214407da73f940f6ca29357a3bd876fa",
          "message": "[otap-dataflow] Allow debug processor to output via a secondary outport (#1171)\n\nAdded a output field to the DebugProcessor allowing the user to select\nwhether to output to `console`, `outport` or `file`\n\nAdded a OutputMode enum for the output setting in the Config and a\nDebugOutput struct to handle the output flow\n\n``` rust\n#[derive(Debug, Clone, Deserialize)]\n#[serde(untagged)]\npub enum OutputMode {\n    Console,\n    Outport(Vec<PortName>),\n    File(String),\n}\n\n/// struct that handles the logic for sending data to console or out_ports\npub struct DebugOutput {\n    writer: Option<OutputWriter>,\n    ports: Option<Vec<PortName>>,\n    effect_handler: local::EffectHandler<OtapPdata>,\n}\n```\n### Output\n\nBy default the debug processor will write the output to the console but\nyou can configure it to write to a file or to send the data to another\npipeline node, you can see how to configure them below.\n\n#### Output to file\n\n```yaml\nconfig:\n   verbosity: normal\n   output: file_name.txt\n```\n\nIn this config the debug-processor will write to a file named\n`file_name.txt`\nit will append to the file rather than overwriting\n\n#### Output to pipeline node\n\n```yaml\n  debug:\n    kind: processor\n    plugin_urn: \"urn:otel:debug:processor\"\n    out_ports:\n      out_port:\n        destinations:\n          - noop\n        dispatch_strategy: round_robin\n      out_port2:\n        destinations:\n          - some_node\n        dispatch_strategy: round_robin\n    config:\n      verbosity: basic\n      output:\n        - out_port2\n```\n\nIn this config we create a processor with multiple out_ports.\nIn the config setting we tell the debug-processor to use `out_port2`\nwhich will send data to another node named `some_node`",
          "timestamp": "2025-10-02T23:31:37Z",
          "tree_id": "f129d1c721179044e007e0087557052a012d2ccf",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/55aaa48e214407da73f940f6ca29357a3bd876fa"
        },
        "date": 1759448388146,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 730333.3333333334,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 21910000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 21910000,
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
            "value": 6.63,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 133.07,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 156.19,
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
            "value": 5.76,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 6.94,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 167.82,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 198.02,
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
          "id": "ced10f89aa98d68bae681ffb16060b3ed176705f",
          "message": "[perf][otel-arrow-rust] Optimize `produce_bar` method used in OTAP Exporter (#1217)\n\n## Changes\n- Remove `HashMap` overhead- no need of repeated `HashMap` lookups and\nretain operations\n- Since the 28 `ArrowPayloadTypes` are known beforehand, we can use a\nfixed array of 28 entries to track the `StreamProducer`s",
          "timestamp": "2025-10-03T22:09:47Z",
          "tree_id": "18990df74bc32b8e7ca5d7b4d4ab125f6020f94a",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/ced10f89aa98d68bae681ffb16060b3ed176705f"
        },
        "date": 1759531247421,
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
            "value": 5.6,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 6.72,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 130.21,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 151.24,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 735166.6666666666,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 22055000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 22055000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-avg",
            "value": 5.56,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 6.8,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 154.63,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 188.26,
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
          "id": "cd54f77cb39afc3ac16d3b9c199b36e63777d039",
          "message": "[otap-df-otap] Update Syslog CEF parser (#1224)\n\n## Changes\n- Update Syslog and CEF parser to access slice items safely by\nvalidating indices are in bounds before accessing the,\n- Add tests to ensure that the parsers don't panic with edge case inputs\n- Add support for handling escape characters in RFC 5424 structured data\n\n---------\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2025-10-06T15:02:38Z",
          "tree_id": "fda5d2fa667a5964610dccf0f1f3cc8eec4c8fca",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/cd54f77cb39afc3ac16d3b9c199b36e63777d039"
        },
        "date": 1759763509614,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 744000,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 22320000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 22320000,
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
            "value": 6.67,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 128.73,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 155.4,
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
            "value": 5.76,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 6.79,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 164.69,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 197.92,
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
          "id": "9bb833f512d2d20d13e9131dc00415568a4e26f6",
          "message": "fix otlp exporter test flake (#1230)\n\nCloses: https://github.com/open-telemetry/otel-arrow/issues/1231\n\nTry to fix test failure observed here: \n\nhttps://github.com/open-telemetry/otel-arrow/actions/runs/18234929441/job/51926734884\n\nThe issue here was that, when we were asserting that the \"logs_failed\"\nmetric was incremented after we send a request which could not be\nserviced, we'd send the pdata, then send the `CollectTelemetry` pdata\nmessage, then wait to receive telemetry on the metrics reporter channel.\nThe problem with this is that the channel that gets fed into the\nexporter biases control messages, so in some cases the exporter would\nsee the `CollectTelemetry` message first. When this happens, there could\nbe no telemetry to report, which means nothing would be sent on the\npdata reporter channel and the test would deadlock.\n\nBecause the test driver can't reliably control the order in which the\nexporter sees these messages two messages in the failure case, we change\nthe test to just check at the of the pdata sequence and ensure we saw\nthe correct number of exported & failed messages. This is reliable\nbecause before checking this, we send a pdata message that should\nsucceed and we can wait on the `req_receiver` ensure the client saw and\nsend this message before we send the `CollectTelemetry` message.\n\nI ran the test in a loop 50x on this commit\nhttps://github.com/open-telemetry/otel-arrow/pull/1230/commits/72e30d114daaa71e4918e53a486448cf3e1e426a\nto ensure it passes reliably.",
          "timestamp": "2025-10-06T16:17:29Z",
          "tree_id": "edfdb5f763e8aca89e1a7095b4dc717a3dbbb70f",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/9bb833f512d2d20d13e9131dc00415568a4e26f6"
        },
        "date": 1759767966834,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 740000,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 22200000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 22200000,
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
            "value": 6.61,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 122.14,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 146.32,
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
            "value": 5.67,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 6.8,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 167.99,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 193.09,
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
          "id": "dfec562fc0450f5a647a7d6a4d61f7b2fb49f525",
          "message": "[otap-dataflow]Debug processor update outport to send correct data (#1220)\n\nUpdated the DebugOutput trait, added additional functions\n```rust\n#[async_trait(?Send)]\npub trait DebugOutput {\n    async fn output_message(&mut self, message: &str) -> Result<(), Error>;\n    async fn output_metrics(&mut self, metric_request: MetricsData) -> Result<(), Error>;\n    async fn output_traces(&mut self, trace_request: TracesData) -> Result<(), Error>;\n    async fn output_logs(&mut self, log_request: LogsData) -> Result<(), Error>;\n    fn is_basic(&self) -> bool;\n}\n```\n\n\nDebugOutputWriter now has a marshaler and display_mode field\n\n```rust\npub struct DebugOutputWriter {\n    writer: Box<dyn AsyncWrite + Unpin>,\n    processor_id: NodeId,\n    marshaler: Option<Box<dyn ViewMarshaler>>,\n    display_mode: DisplayMode,\n}\n```\nDebugOutputPorts has a display_mode field\n\n\n```rust\npub struct DebugOutputPorts {\n    ports: Vec<PortName>,\n    display_mode: DisplayMode,\n    effect_handler: local::EffectHandler<OtapPdata>,\n}\n```",
          "timestamp": "2025-10-06T19:30:50Z",
          "tree_id": "3039c6254db4cf9c3bc7ee730ad19c70ae5e1970",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/dfec562fc0450f5a647a7d6a4d61f7b2fb49f525"
        },
        "date": 1759779513490,
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
            "value": 5.61,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 6.7,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 132.04,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 157.35,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 740666.6666666666,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 22220000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 22220000,
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
            "value": 6.74,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 156.51,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 187.74,
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
          "id": "3c53cad4897643360db79824e71ac0e818ae7c21",
          "message": "Use optimized `append_..._n` methods in syslog receiver when constructing OTAP record (#1196)\n\nCloses #879 \n\nUse the \"append_..._n\" methods in the syslog receiver when constructing\nthe arrow records. These methods have some nice performance\noptimizations:\n- when appending a non-null value if the underlying builder is a\ndictionary array, the builder is able to append the value multiple times\nwithout doing multiple key lookups. For example:\n\nhttps://github.com/apache/arrow-rs/blob/0737c61e76057b127312dd8058887649ece702b8/arrow-array/src/builder/generic_bytes_dictionary_builder.rs#L304-L316\n\n- when the value is null, we can use the optimized code path to append\nmultiple nulls to the null buffer:\n\nhttps://github.com/apache/arrow-rs/blob/0737c61e76057b127312dd8058887649ece702b8/arrow-buffer/src/builder/boolean.rs#L193\n\nBenchmark results:\n```\narrow_batch_creation/rfc3164_arrow_batch_100_msgs\n                        time:   [82.608 µs 82.774 µs 82.950 µs]\n                        change: [−3.9462% −3.4305% −2.9199%] (p = 0.00 < 0.05)\n                        Performance has improved.\n                        \narrow_batch_creation/rfc5424_arrow_batch_100_msgs\n                        time:   [46.139 µs 46.238 µs 46.338 µs]\n                        change: [−4.2893% −3.9924% −3.7179%] (p = 0.00 < 0.05)\n                        Performance has improved.\n                        \narrow_batch_creation/cef_arrow_batch_100_msgs\n                        time:   [40.973 µs 41.141 µs 41.308 µs]\n                        change: [−3.8351% −3.3433% −2.8615%] (p = 0.00 < 0.05)\n                        Performance has improved.\n```",
          "timestamp": "2025-10-06T19:44:51Z",
          "tree_id": "4b294851180d2dc74e2e24334e6380522eb428b5",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/3c53cad4897643360db79824e71ac0e818ae7c21"
        },
        "date": 1759780341466,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 729166.6666666666,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 21875000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 21875000,
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
            "value": 6.71,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 121.89,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 139.62,
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
            "value": 5.71,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 6.67,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 165.55,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 196.82,
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
          "id": "452821e455b16e9a397a09d299340e197eb91571",
          "message": "handle server not ready when starting OTAP Exporter (#1233)\n\ncloses #1096 \n\nFixes issue where the `start` method of the exporter would exit with\nerror immediately if the server identified by config.grpc_endpoint is\nnot ready",
          "timestamp": "2025-10-06T23:28:36Z",
          "tree_id": "3ed520f537ea59bcbb8f199a3782798fdd25b7cf",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/452821e455b16e9a397a09d299340e197eb91571"
        },
        "date": 1759795384406,
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
            "value": 5.68,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 6.83,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 138.4,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 162.61,
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
            "value": 5.77,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 6.76,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 159.8,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 188.78,
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
          "id": "be03b440520b37b00032808ab295127ce5c8ed7d",
          "message": "Update repo codecov to allow for fluctuation (#1241)\n\nCurrently the codecov status check tends to transiently mark builds red\ndue to very small fluctuation in coverage percentages.\n\nThe best examples of this are in non-code PRs like updating github\nworkflow hash versions or release version updates:\n\n-\nhttps://app.codecov.io/gh/open-telemetry/otel-arrow/pull/1131/indirect-changes\n-\nhttps://app.codecov.io/gh/open-telemetry/otel-arrow/pull/1202/indirect-changes\n-\nhttps://app.codecov.io/gh/open-telemetry/otel-arrow/pull/1192/indirect-changes\n\nIt seems there may be some flakiness based on fake data generation - it\nis shown under 'indirect changes' for these PRs:\n\n<img width=\"3218\" height=\"569\" alt=\"image\"\nsrc=\"https://github.com/user-attachments/assets/8e25ba75-df2c-4387-9b89-de3f5aa9258c\"\n/>\n\nThis change:\n\n- allows `0.1%` fluctuation in overall repo coverage to avoid the small\n+- percentage.\n- fixes a small misconfiguration in the `component_management` section\n(`ignore` isn't a valid key there for path scoping)",
          "timestamp": "2025-10-07T18:41:35Z",
          "tree_id": "ce51a93e7e93c5c906caac14441d91308edac942",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/be03b440520b37b00032808ab295127ce5c8ed7d"
        },
        "date": 1759862961053,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 728500,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 21855000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 21855000,
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
            "value": 6.47,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 133.22,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 154.38,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 730666.6666666666,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 21920000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 21920000,
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
            "value": 6.74,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 159.75,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 184.59,
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
          "id": "9643cfcb719f532b8de4c6b693a6893255843b07",
          "message": "Update all patch versions (#1239)\n\n> [!NOTE]\n> Mend has cancelled [the proposed\nrenaming](https://redirect.github.com/renovatebot/renovate/discussions/37842)\nof the Renovate GitHub app being renamed to `mend[bot]`.\n> \n> This notice will be removed on 2025-10-07.\n\n<hr>\n\nThis PR contains the following updates:\n\n| Package | Change | Age | Confidence |\n|---|---|---|---|\n| [grpcio](https://grpc.io)\n([source](https://redirect.github.com/grpc/grpc)) | `==1.75.0` ->\n`==1.75.1` |\n[![age](https://developer.mend.io/api/mc/badges/age/pypi/grpcio/1.75.1?slim=true)](https://docs.renovatebot.com/merge-confidence/)\n|\n[![confidence](https://developer.mend.io/api/mc/badges/confidence/pypi/grpcio/1.75.0/1.75.1?slim=true)](https://docs.renovatebot.com/merge-confidence/)\n|\n| [pandas](https://redirect.github.com/pandas-dev/pandas) | `==2.3.2` ->\n`==2.3.3` |\n[![age](https://developer.mend.io/api/mc/badges/age/pypi/pandas/2.3.3?slim=true)](https://docs.renovatebot.com/merge-confidence/)\n|\n[![confidence](https://developer.mend.io/api/mc/badges/confidence/pypi/pandas/2.3.2/2.3.3?slim=true)](https://docs.renovatebot.com/merge-confidence/)\n|\n| [pydantic](https://redirect.github.com/pydantic/pydantic)\n([changelog](https://docs.pydantic.dev/latest/changelog/)) | `==2.11.9`\n-> `==2.11.10` |\n[![age](https://developer.mend.io/api/mc/badges/age/pypi/pydantic/2.11.10?slim=true)](https://docs.renovatebot.com/merge-confidence/)\n|\n[![confidence](https://developer.mend.io/api/mc/badges/confidence/pypi/pydantic/2.11.9/2.11.10?slim=true)](https://docs.renovatebot.com/merge-confidence/)\n|\n| [pyyaml](https://pyyaml.org/)\n([source](https://redirect.github.com/yaml/pyyaml)) | `==6.0.2` ->\n`==6.0.3` |\n[![age](https://developer.mend.io/api/mc/badges/age/pypi/pyyaml/6.0.3?slim=true)](https://docs.renovatebot.com/merge-confidence/)\n|\n[![confidence](https://developer.mend.io/api/mc/badges/confidence/pypi/pyyaml/6.0.2/6.0.3?slim=true)](https://docs.renovatebot.com/merge-confidence/)\n|\n\n---\n\n### Release Notes\n\n<details>\n<summary>grpc/grpc (grpcio)</summary>\n\n###\n[`v1.75.1`](https://redirect.github.com/grpc/grpc/releases/tag/v1.75.1)\n\n[Compare\nSource](https://redirect.github.com/grpc/grpc/compare/v1.75.0...v1.75.1)\n\nThis is release gRPC Core 1.75.1 (gemini).\n\nFor gRPC documentation, see [grpc.io](https://grpc.io/). For previous\nreleases, see\n[Releases](https://redirect.github.com/grpc/grpc/releases).\n\nThis release contains refinements, improvements, and bug fixes.\n\n##### What's Changed\n\n##### Python\n\n- Release grpcio wheels with Python 3.14 support\n([#&#8203;40403](https://redirect.github.com/grpc/grpc/issues/40403))\n- Asyncio: fixes grpc shutdown race condition occurring during python\ninterpreter finalizations.\n([#&#8203;40447](https://redirect.github.com/grpc/grpc/issues/40447))\n- This also addresses previously reported issues with empty error\nmessage on Python interpreter exit (`Error in sys.excepthook:`/`Original\nexception was:` empty):\n[#&#8203;36655](https://redirect.github.com/grpc/grpc/issues/36655),\n[#&#8203;38679](https://redirect.github.com/grpc/grpc/issues/38679),\n[#&#8203;33342](https://redirect.github.com/grpc/grpc/issues/33342)\n- Python 3.14: preserve current behavior when using grpc.aio async\nmethods outside of a running event loop.\n([#&#8203;40750](https://redirect.github.com/grpc/grpc/issues/40750))\n- Note: using async methods outside of a running event loop is\n[discouraged](https://docs.python.org/3.14/library/asyncio-policy.html#asyncio-policies)\nby Python, and **will be deprecated in future gRPC releases**. Please\nuse the\n[asyncio.run()](https://docs.python.org/3.14/library/asyncio-runner.html#asyncio.run)\nfunction (or\n[asyncio.Runner](https://docs.python.org/3.14/library/asyncio-runner.html#asyncio.Runner)\nfor custom loop factories). For interactive mode, use dedicated [asyncio\nREPL](https://docs.python.org/3/library/asyncio.html#asyncio-cli):\n`python -m asyncio`.\n\n**Full Changelog**:\n<https://github.com/grpc/grpc/compare/v1.75.0...v1.75.1>\n\n</details>\n\n<details>\n<summary>pandas-dev/pandas (pandas)</summary>\n\n###\n[`v2.3.3`](https://redirect.github.com/pandas-dev/pandas/releases/tag/v2.3.3):\nPandas 2.3.3\n\n[Compare\nSource](https://redirect.github.com/pandas-dev/pandas/compare/v2.3.2...v2.3.3)\n\nWe are pleased to announce the release of pandas 2.3.3.\nThis release includes some improvements and fixes to the future string\ndata type (preview feature for the upcoming pandas 3.0). We recommend\nthat all users upgrade to this version.\n\nSee the [full\nwhatsnew](https://pandas.pydata.org/pandas-docs/version/2.3/whatsnew/v2.3.3.html)\nfor a list of all the changes.\nPandas 2.3.3 supports Python 3.9 and higher, and is the first release to\nsupport Python 3.14.\n\nThe release will be available on the conda-forge channel:\n\n```\nconda install pandas --channel conda-forge\n```\n\nOr via PyPI:\n\n```\npython3 -m pip install --upgrade pandas\n```\n\nPlease report any issues with the release on the [pandas issue\ntracker](https://redirect.github.com/pandas-dev/pandas/issues).\n\nThanks to all the contributors who made this release possible.\n\n</details>\n\n<details>\n<summary>pydantic/pydantic (pydantic)</summary>\n\n###\n[`v2.11.10`](https://redirect.github.com/pydantic/pydantic/compare/v2.11.9...v2.11.10)\n\n[Compare\nSource](https://redirect.github.com/pydantic/pydantic/compare/v2.11.9...v2.11.10)\n\n</details>\n\n<details>\n<summary>yaml/pyyaml (pyyaml)</summary>\n\n###\n[`v6.0.3`](https://redirect.github.com/yaml/pyyaml/releases/tag/6.0.3)\n\n[Compare\nSource](https://redirect.github.com/yaml/pyyaml/compare/6.0.2...6.0.3)\n\n#### What's Changed\n\n- Support for Python 3.14 and free-threading (experimental).\n\n**Full Changelog**:\n<https://github.com/yaml/pyyaml/compare/6.0.2...6.0.3>\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - \"before 8am every weekday\" (UTC),\nAutomerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n👻 **Immortal**: This PR will be recreated if closed unmerged. Get\n[config\nhelp](https://redirect.github.com/renovatebot/renovate/discussions) if\nthat's undesired.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0MS4xMzEuOSIsInVwZGF0ZWRJblZlciI6IjQxLjEzMS45IiwidGFyZ2V0QnJhbmNoIjoibWFpbiIsImxhYmVscyI6WyJkZXBlbmRlbmNpZXMiXX0=-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: Joshua MacDonald <jmacd@users.noreply.github.com>",
          "timestamp": "2025-10-07T19:51:25Z",
          "tree_id": "1d3759aac0c71f9006e0242a024585e3a65234eb",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/9643cfcb719f532b8de4c6b693a6893255843b07"
        },
        "date": 1759867214467,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 732166.6666666666,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 21965000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 21965000,
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
            "value": 6.64,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 131.37,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 155.82,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 740333.3333333334,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 22210000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 22210000,
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
            "value": 6.77,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 165.18,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 197.93,
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
          "id": "0beda4d274e899773a68eb539f9c721d8912a228",
          "message": "Parquet exporter: add internal telemetry (#1219)\n\n- Add pdata counters (consumed/exported/failed) and IO counters (files\ncreated/closed, rows written, flush reasons, requeues, age checks).\n- Start a 1s telemetry timer that triggers non-blocking metrics\ncollection via CollectTelemetry\n- Return WriteStats from writer and plumb into exporter to record\nmetrics\n- Add tests for telemetry timer and metrics reporting",
          "timestamp": "2025-10-07T20:35:57Z",
          "tree_id": "bf7b57c222b479c7c18b1f0e2e581dce43547433",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/0beda4d274e899773a68eb539f9c721d8912a228"
        },
        "date": 1759869826726,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 731500,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 21945000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 21945000,
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
            "value": 6.68,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 129.03,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 149.34,
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
            "value": 5.66,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 6.7,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 159.04,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 191.77,
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
          "id": "0b8a7b4ea7cacc1164279c42d4c3f2a0b3392a1e",
          "message": "Pipeline controller: Delayed data support (#1234)\n\nPart of #888 \n\nPart of #783 \n\nPart of #1245 \n\nAdds a \"delay data\" pipeline control message.\nAdds a delayed-data heap in the pipeline controller. \nAdds a \"delayed data\" node control message.\nTests delayed-data delivery.\n\n---------\n\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2025-10-07T21:15:43Z",
          "tree_id": "794bc61a7e629bb2f5679330d43ee14e04d6cec6",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/0b8a7b4ea7cacc1164279c42d4c3f2a0b3392a1e"
        },
        "date": 1759876889979,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 717000,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 21510000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 21510000,
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
            "value": 6.58,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 132.57,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 154.77,
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
            "value": 5.84,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 6.98,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 166.76,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 192.7,
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
          "id": "50bc1ee5c35df7aafa94e58bc5f53c9e0f07aa29",
          "message": "Update dependency duckdb to v1.4.1 (#1248)\n\nThis PR contains the following updates:\n\n| Package | Change | Age | Confidence |\n|---|---|---|---|\n| [duckdb](https://redirect.github.com/duckdb/duckdb-python)\n([changelog](https://redirect.github.com/duckdb/duckdb/releases)) |\n`==1.4.0` -> `==1.4.1` |\n[![age](https://developer.mend.io/api/mc/badges/age/pypi/duckdb/1.4.1?slim=true)](https://docs.renovatebot.com/merge-confidence/)\n|\n[![confidence](https://developer.mend.io/api/mc/badges/confidence/pypi/duckdb/1.4.0/1.4.1?slim=true)](https://docs.renovatebot.com/merge-confidence/)\n|\n\n---\n\n### Release Notes\n\n<details>\n<summary>duckdb/duckdb-python (duckdb)</summary>\n\n###\n[`v1.4.1`](https://redirect.github.com/duckdb/duckdb-python/blob/HEAD/CHANGELOG.md#v141)\n\n[Compare\nSource](https://redirect.github.com/duckdb/duckdb-python/compare/v1.4.0...v1.4.1)\n\n**DuckDB Core**: v1.4.1\n\n##### Bug Fixes\n\n- **ADBC Driver**: Fixed ADBC driver implementation\n([#&#8203;81](https://redirect.github.com/duckdb/duckdb-python/issues/81))\n- **SQLAlchemy compatibility**: Added `__hash__` method overload\n([#&#8203;61](https://redirect.github.com/duckdb/duckdb-python/issues/61))\n- **Error Handling**: Reset PyErr before throwing Python exceptions\n([#&#8203;69](https://redirect.github.com/duckdb/duckdb-python/issues/69))\n- **Polars Lazyframes**: Fixed Polars expression pushdown\n([#&#8203;102](https://redirect.github.com/duckdb/duckdb-python/issues/102))\n\n##### Code Quality Improvements & Developer Experience\n\n- **MyPy Support**: MyPy is functional again and better integrated with\nthe dev workflow\n- **Stubs**: Re-created and manually curated stubs for the binary\nextension\n- **Type Shadowing**: Deprecated `typing` and `functional` modules\n- **Linting & Formatting**: Comprehensive code quality improvements with\nRuff\n- **Type Annotations**: Added missing overloads and improved type\ncoverage\n- **Pre-commit Integration**: Added ruff, clang-format, cmake-format and\nmypy configs\n- **CI/CD**: Added code quality workflow\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - \"before 8am every weekday\" (UTC),\nAutomerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0MS4xMzEuOSIsInVwZGF0ZWRJblZlciI6IjQxLjEzMS45IiwidGFyZ2V0QnJhbmNoIjoibWFpbiIsImxhYmVscyI6WyJkZXBlbmRlbmNpZXMiXX0=-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2025-10-08T15:10:38Z",
          "tree_id": "99b47c25eec62140aa323b690716db8240dbfbf3",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/50bc1ee5c35df7aafa94e58bc5f53c9e0f07aa29"
        },
        "date": 1759936742698,
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
            "value": 5.49,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 6.52,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 121.67,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 143,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 739833.3333333334,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 22195000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 22195000,
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
            "value": 6.77,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 167.43,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 198.34,
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
          "id": "b8a0db148bc35d0820ce315f73534ee100c54f4f",
          "message": "delete '=' file (#1254)\n\ndelete an empty file named `=`",
          "timestamp": "2025-10-09T15:18:17Z",
          "tree_id": "0a07c50fd1bf72c4d03b5be7410f4760336a732b",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/b8a0db148bc35d0820ce315f73534ee100c54f4f"
        },
        "date": 1760025784278,
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
            "value": 5.68,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 6.68,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 134.71,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 159.06,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 736666.6666666666,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 22100000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 22100000,
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
            "value": 6.5,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 166.6,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 197.89,
            "unit": "MiB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "mail@danielthank.me",
            "name": "Yenru Chin",
            "username": "danielthank"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "c4d878c8e96d3b4e9b42731e492010e78776b5f6",
          "message": "Fix map schema to id (#1240)\n\nFix #1223 \n\nThe corresponding go logic is\n\nhttps://github.com/open-telemetry/otel-arrow/blob/452821e455b16e9a397a09d299340e197eb91571/go/pkg/arrow/schema.go#L259-L264\n\nHowever, there is no equivalent `KeyType` and `ItemType` function in\nrust so pattern matching them manually.\n\nArrow rust docs:\nhttps://docs.rs/arrow/latest/arrow/datatypes/enum.DataType.html#variant.Map\n\n---------\n\nCo-authored-by: Utkarsh Umesan Pillai <66651184+utpilla@users.noreply.github.com>\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2025-10-09T17:37:49Z",
          "tree_id": "6587e36608f58e687010bcc6825c449b40a92f70",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/c4d878c8e96d3b4e9b42731e492010e78776b5f6"
        },
        "date": 1760031971649,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 721000,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 21630000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 21630000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 5.48,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 6.39,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 127.68,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 149.79,
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
            "value": 5.72,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 6.96,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 165.63,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 200.91,
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
          "id": "8c8c75614f7d77d2596bab6e4dd3ed2e76bf1d9c",
          "message": "Improve serviceability of the engine (status, livez, readyz) (#1207)\n\nThis PR builds an observed state from the events produced by the engine,\npipeline runtimes, and nodes. An observed state is maintained per\npipeline runtime instance (therefore per core), and an aggregated\nversion is also inferred. The detailed changes are listed below:\n- Observed state per pipeline runtime instance\n- Aggregated state per pipeline (across cores)\n- Pipeline groups status endpoint\n- Liveness and readiness probes, livez and readyz endpoints\n- Report errors on the console (temporary, will be replaced by an\nexporter)\n\nNext steps:\n- #1139 \n- https://github.com/open-telemetry/otel-arrow/issues/1235 \n- https://github.com/open-telemetry/otel-arrow/issues/1236\n- https://github.com/open-telemetry/otel-arrow/issues/1237\n\n---------\n\nCo-authored-by: Joshua MacDonald <jmacd@users.noreply.github.com>",
          "timestamp": "2025-10-09T17:58:19Z",
          "tree_id": "3474683253149eb04ee57e00195c12c53a2597ae",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/8c8c75614f7d77d2596bab6e4dd3ed2e76bf1d9c"
        },
        "date": 1760033698329,
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
            "value": 5.55,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 6.63,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 122.28,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 141.11,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 718333.3333333334,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 21550000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 21550000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-avg",
            "value": 5.55,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 6.63,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 167.33,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 204.3,
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
          "id": "98baa296afefb5b8d2204f28cf5f321678c14bea",
          "message": "Complete Ack/Nack control message routing and delivery (#1249)\n\nThis finishes the basic integration of Ack/Nack delivery into the\nengine. Future work will involve modifying all components with an\nAck/Nack interest to subscribe-to and route these correctly.\n\nFixes #498 \nFixes #1244.\n\nAdds DeliverAck, DeliverNack messages to the pipeline controller.\n\nFinishes EffectHandlerCore route_ack, route_nack. Modifies\nerror_exporter and noop_exporter tests for Nack and Ack delivery.\n\nIncorporates https://github.com/open-telemetry/otel-arrow/pull/1247;\ntests RETURN_DATA integration with Ack/Nack delivery.\n\nNoteworthy simplification: route_ack and route_nack return untyped\nErrors, making error handling much easier. We can revisit this, in case\nwe want to handle the data that would otherwise drop when it can't be\nreturned.",
          "timestamp": "2025-10-09T18:24:18Z",
          "tree_id": "5dacf91bfe3c8d6401b9dbddd926917018e29726",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/98baa296afefb5b8d2204f28cf5f321678c14bea"
        },
        "date": 1760035460778,
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
            "value": 5.67,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 6.7,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 129,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 155.76,
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
            "value": 6.77,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 166.54,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 192.14,
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
          "id": "2b7c7547e8e611b0727f5d2cf2c2aadc3884d5ea",
          "message": "[otap-dataflow] Add Sampling (based on zap) to DebugProcessor (#1250)\n\nAdded the SamplingConfig struct to allow the user to configure their\nsampling settings\n\n```rust\n#[derive(Debug, Clone, Copy, Deserialize)]\npub struct SamplingConfig {\n    // number of samples to log initially within the sampling interval\n    pub sampling_initial: u64,\n    // the sampling rate\n    pub sampling_thereafter: u64,\n    // the interval that we sample in, unit is seconds\n    pub sampling_interval: u64,\n}\n```\n\nAdded the Sampler struct to handle the sampling logic for the debug\nprocessor\n\n```rust\n/// The sampler keeps track of the current state, number of msgs seen and next interval\npub struct Sampler {\n    // sampling settings\n    sampling_config: SamplingConfig,\n    // counter for msgs seen\n    msgs_current_interval: u64,\n    next_interval: Instant,\n}\n```\n\nuses the sample function that accepts a closure (in this closure is the\ncall to output the msgs either to console or outport depending on\nconfiguration) to determine whether to log or drop a msg",
          "timestamp": "2025-10-09T21:23:28Z",
          "tree_id": "adf4892a7d22c6279ea89aecfbc43aeecae59c06",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/2b7c7547e8e611b0727f5d2cf2c2aadc3884d5ea"
        },
        "date": 1760047116286,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 735166.6666666666,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 22055000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 22055000,
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
            "value": 6.65,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 114.79,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 137.09,
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
            "value": 168.05,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 199.21,
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
          "id": "36c7123433eb42b19b24388f6e0a20247a9b0ce9",
          "message": "Phase 2 work-in-progress documentation update (#1263)\n\nThis updates a number of key README files to explain and help navigate\nour new Rust-based OTAP Dataflow pipeline.\n\n---------\n\nCo-authored-by: Laurent Quérel <laurent.querel@gmail.com>\nCo-authored-by: Utkarsh Umesan Pillai <66651184+utpilla@users.noreply.github.com>\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2025-10-10T15:25:29Z",
          "tree_id": "cc3a902d4d876fb61a33fc274a9660514cd7012b",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/36c7123433eb42b19b24388f6e0a20247a9b0ce9"
        },
        "date": 1760110480176,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 715666.6666666666,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 21470000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 21470000,
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
            "value": 6.58,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 126.66,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 149.89,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 729166.6666666666,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 21875000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 21875000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-avg",
            "value": 5.57,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 6.77,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 162.34,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 202.23,
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
          "id": "f6bbee611ae26f6a0438e43c53f17929c937066e",
          "message": "[query-engine] Relax restrictions in KQL summarize expressions (#1262)\n\n## Changes\n\n* Allow aggregate and group by expression without explicit assignment in\nKQL summarize expressions\n\n## Details\n\nToday you can do:\n* `summarize by Timestamp = bin(Timestamp, 1m)`\n* `summarize Count = count()`\n\nBut you will get an error if you do:\n* `summarize by bin(Timestamp, 1m)`\n* `summarize count()`\n\nThis PR makes the second two examples work. In the first case the name\n`Timestamp` will be used and in the second case `count`.\n\nThis is done by giving the KQL parser some logic to determine the\nidentifier for any given scalar expression. Note: We could probably use\nthis in extend\\project* cases too but I plan to look at that in a\nfollow-up.\n\nFixes #1243\n\n---------\n\nCo-authored-by: Drew Relmas <drewrelmas@gmail.com>",
          "timestamp": "2025-10-10T19:35:30Z",
          "tree_id": "0c4d83f671067aa33da17d9c5c3291a7e54d7a7a",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/f6bbee611ae26f6a0438e43c53f17929c937066e"
        },
        "date": 1760125552714,
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
            "value": 125.04,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 146.69,
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
            "value": 5.73,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 6.77,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 167.7,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 197.71,
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
          "id": "346319a8227aeec933c95ed2b955c3974f9309b6",
          "message": "[otap-df-otap] Add support for CEF messages with Syslog headers (#1264)\n\nAddress #1073\n\n## Changes\n- Add support for parsing CEF messages with Syslog header\n- Note that we still support parsing raw CEF messages\n\n\n### Examples:\n\n#### Raw CEF\n```CEF:0|Security|threatmanager|1.0|100|worm successfully stopped|10|src=10.0.0.1 dst=2.1.2.2 spt=1232```\n\n#### CEF with Syslog header (This is a partial header with RFC 3164, example is from the [spec](https://www.microfocus.com/documentation/arcsight/arcsight-smartconnectors-8.3/cef-implementation-standard/Content/CEF/Chapter%201%20What%20is%20CEF.htm))\n```Sep 29 08:26:10 host CEF:1|Security|threatmanager|1.0|100|worm\nsuccessfully stopped|10|src=10.0.0.1 dst=2.1.2.2 spt=1232```",
          "timestamp": "2025-10-10T20:12:13Z",
          "tree_id": "c141a27d0fa0a176fef2838b3a65c3096866b471",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/346319a8227aeec933c95ed2b955c3974f9309b6"
        },
        "date": 1760127645074,
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
            "value": 5.63,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 6.66,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 125.15,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 149.73,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 734833.3333333334,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 22045000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 22045000,
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
            "value": 6.93,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 149.28,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 180.54,
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
          "id": "f26769706c0ba307d5d39ce01ba96563a9615fed",
          "message": "[query-engine] Validate unique identifiers in KQL summarize statements (#1267)\n\nRelates to #1262\n\n## Changes\n\n* Validate unique identifiers in KQL summarize statements",
          "timestamp": "2025-10-10T20:49:21Z",
          "tree_id": "da73c29921d35470af3885942da66e1f276799bb",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/f26769706c0ba307d5d39ce01ba96563a9615fed"
        },
        "date": 1760129858287,
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
            "value": 125.42,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 146.46,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 728000,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 21840000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 21840000,
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
            "value": 6.64,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 159.15,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 187,
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
          "id": "406fd4d3ec36305323e4a096559d421156b762f7",
          "message": "Update docker.io/rust Docker tag to v1.90 (#1271)\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n|---|---|---|---|\n| docker.io/rust | stage | minor | `1.89` -> `1.90` |\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - \"before 8am on Monday\" (UTC),\nAutomerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0MS4xNDMuMSIsInVwZGF0ZWRJblZlciI6IjQxLjE0My4xIiwidGFyZ2V0QnJhbmNoIjoibWFpbiIsImxhYmVscyI6WyJkZXBlbmRlbmNpZXMiXX0=-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2025-10-13T21:41:01Z",
          "tree_id": "49e2d5112396b45f53b0671cbbef96b68933bd6c",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/406fd4d3ec36305323e4a096559d421156b762f7"
        },
        "date": 1760392154241,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 723833.3333333334,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 21715000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 21715000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 5.47,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 6.55,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 120.64,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 141.9,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 731333.3333333334,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 21940000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 21940000,
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
            "value": 6.88,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 167.85,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 195.08,
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
          "id": "35a1d8638f0938c95ea53fea10812443b3dd9846",
          "message": "Centralize workspace dependencies (x9) (#1274)\n\nDiscussed in last week's SIG. In otap-dataflow/crates/* except /otap,\nthe dependencies are now 100% workspace-defined.\nIn many cases, the features are already exposed. I created a new block\nin the workspace Cargo.toml, will re-alphabetize these later.\n\nPart of #1218",
          "timestamp": "2025-10-14T00:34:45Z",
          "tree_id": "4a59c24c9f11de7a0da568d92ac19b9d2a9f53e0",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/35a1d8638f0938c95ea53fea10812443b3dd9846"
        },
        "date": 1760402585173,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 728500,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 21855000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 21855000,
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
            "value": 6.55,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 136.75,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 161.54,
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
            "value": 5.72,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 6.83,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 162.63,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 188.84,
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
          "id": "c66ab978a75877d51cc7a4e8421de1f55523e9bb",
          "message": "Centralize workspace dependencies (otap-df-otap) (#1275)\n\nFollows #1274.\nPart of #1218",
          "timestamp": "2025-10-14T15:12:32Z",
          "tree_id": "efe42e82202c260c2f15ba47e5aec22e05f9760c",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/c66ab978a75877d51cc7a4e8421de1f55523e9bb"
        },
        "date": 1760455535011,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 736333.3333333334,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 22090000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 22090000,
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
            "value": 6.72,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 124.4,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 147.62,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 734500,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 22035000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 22035000,
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
            "value": 6.8,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 158.5,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 188.66,
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
          "id": "acdf88e53b55a2b31d238610a11b32ea7d9f5b59",
          "message": "Centralize workspace dependencies (benchmarks) (#1277)\n\nPart of #1218",
          "timestamp": "2025-10-14T15:33:28Z",
          "tree_id": "58629694b79b4a5e0903477ba3902357cabda984",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/acdf88e53b55a2b31d238610a11b32ea7d9f5b59"
        },
        "date": 1760456473745,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 727333.3333333334,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 21820000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 21820000,
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
            "value": 6.46,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 118.93,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 145.73,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 732000,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 21960000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 21960000,
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
            "value": 6.9,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 166.48,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 202.32,
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
          "id": "9182e56dd3de8a9e445302273de6ca7ad8a94f5f",
          "message": "Update dependency pydantic to v2.12.2 (#1280)\n\nThis PR contains the following updates:\n\n| Package | Change | Age | Confidence |\n|---|---|---|---|\n| [pydantic](https://redirect.github.com/pydantic/pydantic)\n([changelog](https://docs.pydantic.dev/latest/changelog/)) | `==2.12.0`\n-> `==2.12.2` |\n[![age](https://developer.mend.io/api/mc/badges/age/pypi/pydantic/2.12.2?slim=true)](https://docs.renovatebot.com/merge-confidence/)\n|\n[![confidence](https://developer.mend.io/api/mc/badges/confidence/pypi/pydantic/2.12.0/2.12.2?slim=true)](https://docs.renovatebot.com/merge-confidence/)\n|\n\n---\n\n### Release Notes\n\n<details>\n<summary>pydantic/pydantic (pydantic)</summary>\n\n###\n[`v2.12.2`](https://redirect.github.com/pydantic/pydantic/releases/tag/v2.12.2):\n2025-10-14\n\n#### v2.12.2 (2025-10-14)\n\n##### What's Changed\n\n##### Fixes\n\n- Release a new `pydantic-core` version, as a corrupted CPython 3.10\n`manylinux2014_aarch64` wheel got uploaded\n([pydantic-core#1843](https://redirect.github.com/pydantic/pydantic-core/pull/1843)).\n- Fix issue with recursive generic models with a parent model class by\n[@&#8203;Viicos](https://redirect.github.com/Viicos) in\n[#&#8203;12398](https://redirect.github.com/pydantic/pydantic/pull/12398)\n\n**Full Changelog**:\n<https://github.com/pydantic/pydantic/compare/v2.12.1...v2.12.2>\n\n###\n[`v2.12.1`](https://redirect.github.com/pydantic/pydantic/releases/tag/v2.12.1):\n2025-10-13\n\n[Compare\nSource](https://redirect.github.com/pydantic/pydantic/compare/v2.12.0...v2.12.1)\n\n#### v2.12.1 (2025-10-13)\n\n[GitHub\nrelease](https://redirect.github.com/pydantic/pydantic/releases/tag/v2.12.1)\n\n##### What's Changed\n\nThis is the first 2.12 patch release, addressing most (but not all yet)\nregressions from the initial 2.12.0 release.\n\n##### Fixes\n\n- Do not evaluate annotations when inspecting validators and serializers\nby [@&#8203;Viicos](https://redirect.github.com/Viicos) in\n[#&#8203;12355](https://redirect.github.com/pydantic/pydantic/pull/12355)\n- Make sure `None` is converted as `NoneType` in Python 3.14 by\n[@&#8203;Viicos](https://redirect.github.com/Viicos) in\n[#&#8203;12370](https://redirect.github.com/pydantic/pydantic/pull/12370)\n- Backport V1 runtime warning when using Python 3.14 by\n[@&#8203;Viicos](https://redirect.github.com/Viicos) in\n[#&#8203;12367](https://redirect.github.com/pydantic/pydantic/pull/12367)\n- Fix error message for invalid validator signatures by\n[@&#8203;Viicos](https://redirect.github.com/Viicos) in\n[#&#8203;12366](https://redirect.github.com/pydantic/pydantic/pull/12366)\n- Populate field name in `ValidationInfo` for validation of default\nvalue by [@&#8203;Viicos](https://redirect.github.com/Viicos) in\n[pydantic-core#1826](https://redirect.github.com/pydantic/pydantic-core/pull/1826)\n- Encode credentials in `MultiHostUrl` builder by\n[@&#8203;willswire](https://redirect.github.com/willswire) in\n[pydantic-core#1829](https://redirect.github.com/pydantic/pydantic-core/pull/1829)\n- Respect field serializers when using `serialize_as_any` serialization\nflag by [@&#8203;davidhewitt](https://redirect.github.com/davidhewitt)\nin\n[pydantic-core#1829](https://redirect.github.com/pydantic/pydantic-core/pull/1829)\n- Fix various `RootModel` serialization issues by\n[@&#8203;davidhewitt](https://redirect.github.com/davidhewitt) in\n[pydantic-core#1836](https://redirect.github.com/pydantic/pydantic-core/pull/1836)\n\n##### New Contributors\n\n- [@&#8203;willswire](https://redirect.github.com/willswire) made their\nfirst contribution in\n[pydantic-core#1829](https://redirect.github.com/pydantic/pydantic-core/pull/1829)\n\n**Full Changelog**:\n<https://github.com/pydantic/pydantic/compare/v2.12.0...v2.12.1>\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - \"before 8am every weekday\" (UTC),\nAutomerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0MS4xNDMuMSIsInVwZGF0ZWRJblZlciI6IjQxLjE0My4xIiwidGFyZ2V0QnJhbmNoIjoibWFpbiIsImxhYmVscyI6WyJkZXBlbmRlbmNpZXMiXX0=-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2025-10-14T15:43:31Z",
          "tree_id": "43648617aa6c15b9ee0c137e79ae67eccfd0b6e4",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/9182e56dd3de8a9e445302273de6ca7ad8a94f5f"
        },
        "date": 1760457136007,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 726000,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 21780000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 21780000,
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
            "value": 6.58,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 126.87,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 148.7,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 731833.3333333334,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 21955000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 21955000,
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
            "value": 6.84,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 164.14,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 191.33,
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
          "id": "f6074fd8f219dfbc59219ae49a0015368eb29eb4",
          "message": "Improve graceful shutdown mechanism to ensure full metrics flushing and orderly pipeline termination (#1273)\n\n**Note: Many files were modified in this PR, but most of the changes are\nsignature updates.**\n\nThis PR refines the graceful shutdown process of the pipeline engine to\nmake node termination deterministic and data-safe (in particular for the\nmetrics).\n\nThe following diagram describes the process.\n<img width=\"1148\" height=\"735\" alt=\"image\"\nsrc=\"https://github.com/user-attachments/assets/fa2cb359-766e-4ba0-85ec-16c195c1f483\"\n/>\n\nOther changes:\n- The deadline field in the Shutdown control message now represents an\nactual deadline instead of a duration.\n- Local channels now expose an `is_empty` method, consistent with other\nchannel implementations.\n- Simplified some unit tests.\n- Improved TestRuntime to make `otap_exporter` testing more robust.\n\t\nCloses: #1139\n\n---------\n\nCo-authored-by: Joshua MacDonald <jmacd@users.noreply.github.com>",
          "timestamp": "2025-10-14T18:00:59Z",
          "tree_id": "390965036e096950a66a21af819c5a7752c58f4e",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/f6074fd8f219dfbc59219ae49a0015368eb29eb4"
        },
        "date": 1760465372278,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 717666.6666666666,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 21530000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 21530000,
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
            "value": 6.58,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 125.83,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 147.42,
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
            "value": 5.59,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 6.55,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 171.58,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 204,
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
          "id": "e85189d833d64de607ec97265655d116078a5a8e",
          "message": "Rename otap-df-pdata-views to otap-df-pdata::views (#1276)\n\nPart of #1218 \n\nI want the package named otap-df-pdata to relocate most of\notel-arrow-rust. This is the first step.",
          "timestamp": "2025-10-14T21:18:12Z",
          "tree_id": "b9439ea2c50d51a4e9dc47266ca1c5aac39ea5c9",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/e85189d833d64de607ec97265655d116078a5a8e"
        },
        "date": 1760477155387,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 726500,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 21795000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 21795000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 5.48,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 6.43,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 122.44,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 141.71,
            "unit": "MiB"
          },
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
            "value": 5.62,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 6.88,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 156.52,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 184.43,
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
          "id": "457e03256b0b13068043af4a48c20b7c1f84ac41",
          "message": "Sort otap-dataflow workspace dependencies (#1282)\n\nNo changes, only sorting.",
          "timestamp": "2025-10-14T21:31:44Z",
          "tree_id": "8a9b07f26c6804c5a2fec46c3f8d0f934b1b6d1a",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/457e03256b0b13068043af4a48c20b7c1f84ac41"
        },
        "date": 1760478005824,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 716666.6666666666,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 21500000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 21500000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 5.48,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 6.47,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 128.83,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 154.43,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 730666.6666666666,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 21920000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 21920000,
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
            "value": 6.61,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 157.42,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 182.29,
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
          "id": "d3cc6a99b467af09937212696793eee3b8836bae",
          "message": "[query-engine] Add support for defining map schema while allowing undefined keys (#1281)\n\nRequested by @drewrelmas\n\n## Changes\n\n* Add support in KQL Parser for defining map schema while still allowing\nundefined keys\n\n## Details\n\nCurrently if you define schema for a map then any unknown key will yield\n\"key-not-found\" errors. @drewrelmas has a user scenario where he wants\nto define schema for known keys but still allow custom keys to be\nadded/removed/modified. This PR essentially adds a flag\n`allow_undefined_keys` to enable this behavior.\n\nNote: There will be a follow-up PR exposing this in the bridge options.",
          "timestamp": "2025-10-14T23:46:19Z",
          "tree_id": "3931afc2b0c5b89d9fffab67566652109ea1f83d",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/d3cc6a99b467af09937212696793eee3b8836bae"
        },
        "date": 1760486040961,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 727166.6666666666,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 21815000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 21815000,
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
            "value": 126.22,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 147.92,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 735666.6666666666,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 22070000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 22070000,
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
            "value": 6.69,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 157.56,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 185.4,
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
          "id": "4ed8aa6d1fdb70a2bbdd72574f0929d51e654251",
          "message": "Print ACK/NACK in debug_processor at detailed level (#1265)\n\nPart of #1249 \nThis prints the ACK or NACK by after storing the u128 of SystemTime\nmicroseconds as two u64 in the CallData.\nManually tested, e.g., changed fake-debug-noop.yaml to use `detailed`\nlevel, then\n\n```\ncargo run --bin df_engine -- --num-cores=1 --pipeline ./configs/fake-debug-noop.yaml \n```\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2025-10-15T00:47:48Z",
          "tree_id": "37ed568ff25aadcf1c4873c07609febf8d985d2a",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/4ed8aa6d1fdb70a2bbdd72574f0929d51e654251"
        },
        "date": 1760492513812,
        "tool": "customSmallerIsBetter",
        "benches": [
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
            "value": 123.32,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 149.07,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 731500,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 21945000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 21945000,
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
            "value": 7,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 165.38,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 192.15,
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
          "id": "877b2cb3dddef09143c7cb20164fd0ba789dd210",
          "message": "[query-engine] Expose allow_undefined_keys on BridgeOptions (#1286)\n\nRelates to #1281\n\n## Changes\n\n* Allow `allow_undefined_keys` on attributes schema to be set via\n`BridgeOptions`",
          "timestamp": "2025-10-15T17:12:44Z",
          "tree_id": "8e1b90c2eecf38a2918d279a1aff37c694e7aff1",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/877b2cb3dddef09143c7cb20164fd0ba789dd210"
        },
        "date": 1760548874541,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": 733166.6666666666,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": 21995000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 21995000,
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
            "value": 6.58,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 129.35,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 151.66,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": 727333.3333333334,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": 21820000,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 21820000,
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
            "value": 159.31,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 192.83,
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
          "id": "67d9813bd18a4accf723f190de8bf7b735ddbc86",
          "message": "[PerfTest] Update loadgen script to support static message body (#1287)\n\nThis PR updates the python loadgen script to support:\n\n1. Syslog headers as optional or either of the RFC formats (default\nrfc3164, no change to behavior).\n2. Message body may be supplied as a static string (e.g. a CEF or other\nformat message, default is still randomly-generated string).\n3. Renames the \"sent\" metric to \"logs_produced\" for consistency with the\nfake signal generator and easier reporting (no change to the existing\norchestrator.py required - it's looking at the unchanged final logged\nstring, not the metric).\n\nThis will enable additional nightly benchmarks for e.g. CEF parsing, etc\nin a followup PR.",
          "timestamp": "2025-10-15T22:14:04Z",
          "tree_id": "4552aff68287e808783b8121e734c9bc7bcdd1d7",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/67d9813bd18a4accf723f190de8bf7b735ddbc86"
        },
        "date": 1760566915623,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": -0.03333333333333333,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": -1,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 0,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 20.73,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 20.73,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": -0.03333333333333333,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": -1,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 0,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-avg",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 22.35,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 22.51,
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
          "id": "2e2dad73f0b53c44690e7fea357cfb5d5957664a",
          "message": "Update github workflow dependencies (major) (#1290)\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n|---|---|---|---|\n| [actions/setup-node](https://redirect.github.com/actions/setup-node) |\naction | major | `v5.0.0` -> `v6.0.0` |\n|\n[github/codeql-action](https://redirect.github.com/github/codeql-action)\n| action | major | `v3.30.8` -> `v4.30.8` |\n\n---\n\n### Release Notes\n\n<details>\n<summary>actions/setup-node (actions/setup-node)</summary>\n\n###\n[`v6.0.0`](https://redirect.github.com/actions/setup-node/releases/tag/v6.0.0)\n\n[Compare\nSource](https://redirect.github.com/actions/setup-node/compare/v5.0.0...v6.0.0)\n\n#### What's Changed\n\n**Breaking Changes**\n\n- Limit automatic caching to npm, update workflows and documentation by\n[@&#8203;priyagupta108](https://redirect.github.com/priyagupta108) in\n[#&#8203;1374](https://redirect.github.com/actions/setup-node/pull/1374)\n\n**Dependency Upgrades**\n\n- Upgrade ts-jest from 29.1.2 to 29.4.1 and document breaking changes in\nv5 by [@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot]\nin\n[#&#8203;1336](https://redirect.github.com/actions/setup-node/pull/1336)\n- Upgrade prettier from 2.8.8 to 3.6.2 by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;1334](https://redirect.github.com/actions/setup-node/pull/1334)\n- Upgrade actions/publish-action from 0.3.0 to 0.4.0 by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;1362](https://redirect.github.com/actions/setup-node/pull/1362)\n\n**Full Changelog**:\n<https://github.com/actions/setup-node/compare/v5...v6.0.0>\n\n</details>\n\n<details>\n<summary>github/codeql-action (github/codeql-action)</summary>\n\n###\n[`v4.30.8`](https://redirect.github.com/github/codeql-action/releases/tag/v4.30.8)\n\n[Compare\nSource](https://redirect.github.com/github/codeql-action/compare/v4.30.7...v4.30.8)\n\n### CodeQL Action Changelog\n\nSee the [releases\npage](https://redirect.github.com/github/codeql-action/releases) for the\nrelevant changes to the CodeQL CLI and language packs.\n\n#### 4.30.8 - 10 Oct 2025\n\nNo user facing changes.\n\nSee the full\n[CHANGELOG.md](https://redirect.github.com/github/codeql-action/blob/v4.30.8/CHANGELOG.md)\nfor more information.\n\n###\n[`v4.30.7`](https://redirect.github.com/github/codeql-action/releases/tag/v4.30.7)\n\n[Compare\nSource](https://redirect.github.com/github/codeql-action/compare/v3.30.8...v4.30.7)\n\n##### CodeQL Action Changelog\n\nSee the [releases\npage](https://redirect.github.com/github/codeql-action/releases) for the\nrelevant changes to the CodeQL CLI and language packs.\n\n##### 4.30.7 - 06 Oct 2025\n\n- \\[v4+ only] The CodeQL Action now runs on Node.js v24.\n[#&#8203;3169](https://redirect.github.com/github/codeql-action/pull/3169)\n\nSee the full\n[CHANGELOG.md](https://redirect.github.com/github/codeql-action/blob/v4.30.7/CHANGELOG.md)\nfor more information.\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - \"before 8am on Monday\" (UTC),\nAutomerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n👻 **Immortal**: This PR will be recreated if closed unmerged. Get\n[config\nhelp](https://redirect.github.com/renovatebot/renovate/discussions) if\nthat's undesired.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0MS4xNDMuMSIsInVwZGF0ZWRJblZlciI6IjQxLjE0My4xIiwidGFyZ2V0QnJhbmNoIjoibWFpbiIsImxhYmVscyI6WyJkZXBlbmRlbmNpZXMiXX0=-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2025-10-16T15:08:49Z",
          "tree_id": "85e0d70b13b38da2b231b1aecc2937b43c090e79",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/2e2dad73f0b53c44690e7fea357cfb5d5957664a"
        },
        "date": 1760627842443,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": -0.03333333333333333,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": -1,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 0,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 20.59,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 20.59,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": -0.03333333333333333,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": -1,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 0,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-avg",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 22.62,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 22.63,
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
          "id": "ac76786947bcac5a29e688218a68038ec5fd0f83",
          "message": "Update taiki-e/install-action action to v2.62.32 (#1297)\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n|---|---|---|---|\n|\n[taiki-e/install-action](https://redirect.github.com/taiki-e/install-action)\n| action | patch | `v2.62.30` -> `v2.62.32` |\n\n---\n\n### Release Notes\n\n<details>\n<summary>taiki-e/install-action (taiki-e/install-action)</summary>\n\n###\n[`v2.62.32`](https://redirect.github.com/taiki-e/install-action/blob/HEAD/CHANGELOG.md#100---2021-12-30)\n\n[Compare\nSource](https://redirect.github.com/taiki-e/install-action/compare/v2.62.31...v2.62.32)\n\nInitial release\n\n[Unreleased]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.32...HEAD\n\n[2.62.32]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.31...v2.62.32\n\n[2.62.31]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.30...v2.62.31\n\n[2.62.30]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.29...v2.62.30\n\n[2.62.29]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.28...v2.62.29\n\n[2.62.28]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.27...v2.62.28\n\n[2.62.27]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.26...v2.62.27\n\n[2.62.26]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.25...v2.62.26\n\n[2.62.25]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.24...v2.62.25\n\n[2.62.24]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.23...v2.62.24\n\n[2.62.23]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.22...v2.62.23\n\n[2.62.22]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.21...v2.62.22\n\n[2.62.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.20...v2.62.21\n\n[2.62.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.19...v2.62.20\n\n[2.62.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.18...v2.62.19\n\n[2.62.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.17...v2.62.18\n\n[2.62.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.16...v2.62.17\n\n[2.62.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.15...v2.62.16\n\n[2.62.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.14...v2.62.15\n\n[2.62.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.13...v2.62.14\n\n[2.62.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.12...v2.62.13\n\n[2.62.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.11...v2.62.12\n\n[2.62.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.10...v2.62.11\n\n[2.62.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.9...v2.62.10\n\n[2.62.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.8...v2.62.9\n\n[2.62.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.7...v2.62.8\n\n[2.62.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.6...v2.62.7\n\n[2.62.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.5...v2.62.6\n\n[2.62.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.4...v2.62.5\n\n[2.62.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.3...v2.62.4\n\n[2.62.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.2...v2.62.3\n\n[2.62.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.1...v2.62.2\n\n[2.62.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.0...v2.62.1\n\n[2.62.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.13...v2.62.0\n\n[2.61.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.12...v2.61.13\n\n[2.61.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.11...v2.61.12\n\n[2.61.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.10...v2.61.11\n\n[2.61.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.9...v2.61.10\n\n[2.61.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.8...v2.61.9\n\n[2.61.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.7...v2.61.8\n\n[2.61.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.6...v2.61.7\n\n[2.61.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.5...v2.61.6\n\n[2.61.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.4...v2.61.5\n\n[2.61.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.3...v2.61.4\n\n[2.61.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.2...v2.61.3\n\n[2.61.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.1...v2.61.2\n\n[2.61.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.0...v2.61.1\n\n[2.61.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.60.0...v2.61.0\n\n[2.60.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.59.1...v2.60.0\n\n[2.59.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.59.0...v2.59.1\n\n[2.59.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.33...v2.59.0\n\n[2.58.33]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.32...v2.58.33\n\n[2.58.32]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.31...v2.58.32\n\n[2.58.31]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.30...v2.58.31\n\n[2.58.30]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.29...v2.58.30\n\n[2.58.29]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.28...v2.58.29\n\n[2.58.28]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.27...v2.58.28\n\n[2.58.27]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.26...v2.58.27\n\n[2.58.26]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.25...v2.58.26\n\n[2.58.25]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.24...v2.58.25\n\n[2.58.24]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.23...v2.58.24\n\n[2.58.23]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.22...v2.58.23\n\n[2.58.22]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.21...v2.58.22\n\n[2.58.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.20...v2.58.21\n\n[2.58.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.19...v2.58.20\n\n[2.58.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.18...v2.58.19\n\n[2.58.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.17...v2.58.18\n\n[2.58.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.16...v2.58.17\n\n[2.58.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.15...v2.58.16\n\n[2.58.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.14...v2.58.15\n\n[2.58.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.13...v2.58.14\n\n[2.58.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.12...v2.58.13\n\n[2.58.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.11...v2.58.12\n\n[2.58.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.10...v2.58.11\n\n[2.58.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.9...v2.58.10\n\n[2.58.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.8...v2.58.9\n\n[2.58.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.7...v2.58.8\n\n[2.58.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.6...v2.58.7\n\n[2.58.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.5...v2.58.6\n\n[2.58.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.4...v2.58.5\n\n[2.58.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.3...v2.58.4\n\n[2.58.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.2...v2.58.3\n\n[2.58.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.1...v2.58.2\n\n[2.58.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.0...v2.58.1\n\n[2.58.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.8...v2.58.0\n\n[2.57.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.7...v2.57.8\n\n[2.57.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.6...v2.57.7\n\n[2.57.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.5...v2.57.6\n\n[2.57.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.4...v2.57.5\n\n[2.57.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.3...v2.57.4\n\n[2.57.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.2...v2.57.3\n\n[2.57.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.1...v2.57.2\n\n[2.57.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.0...v2.57.1\n\n[2.57.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.24...v2.57.0\n\n[2.56.24]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.23...v2.56.24\n\n[2.56.23]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.22...v2.56.23\n\n[2.56.22]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.21...v2.56.22\n\n[2.56.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.20...v2.56.21\n\n[2.56.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.19...v2.56.20\n\n[2.56.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.18...v2.56.19\n\n[2.56.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.17...v2.56.18\n\n[2.56.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.16...v2.56.17\n\n[2.56.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.15...v2.56.16\n\n[2.56.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.14...v2.56.15\n\n[2.56.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.13...v2.56.14\n\n[2.56.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.12...v2.56.13\n\n[2.56.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.11...v2.56.12\n\n[2.56.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.10...v2.56.11\n\n[2.56.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.9...v2.56.10\n\n[2.56.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.8...v2.56.9\n\n[2.56.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.7...v2.56.8\n\n[2.56.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.6...v2.56.7\n\n[2.56.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.5...v2.56.6\n\n[2.56.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.4...v2.56.5\n\n[2.56.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.3...v2.56.4\n\n[2.56.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.2...v2.56.3\n\n[2.56.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.1...v2.56.2\n\n[2.56.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.0...v2.56.1\n\n[2.56.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.55.4...v2.56.0\n\n[2.55.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.55.3...v2.55.4\n\n[2.55.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.55.2...v2.55.3\n\n[2.55.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.55.1...v2.55.2\n\n[2.55.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.55.0...v2.55.1\n\n[2.55.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.54.3...v2.55.0\n\n[2.54.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.54.2...v2.54.3\n\n[2.54.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.54.1...v2.54.2\n\n[2.54.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.54.0...v2.54.1\n\n[2.54.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.53.2...v2.54.0\n\n[2.53.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.53.1...v2.53.2\n\n[2.53.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.53.0...v2.53.1\n\n[2.53.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.8...v2.53.0\n\n[2.52.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.7...v2.52.8\n\n[2.52.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.6...v2.52.7\n\n[2.52.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.5...v2.52.6\n\n[2.52.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.4...v2.52.5\n\n[2.52.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.3...v2.52.4\n\n[2.52.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.2...v2.52.3\n\n[2.52.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.1...v2.52.2\n\n[2.52.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.0...v2.52.1\n\n[2.52.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.51.3...v2.52.0\n\n[2.51.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.51.2...v2.51.3\n\n[2.51.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.51.1...v2.51.2\n\n[2.51.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.51.0...v2.51.1\n\n[2.51.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.10...v2.51.0\n\n[2.50.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.9...v2.50.10\n\n[2.50.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.8...v2.50.9\n\n[2.50.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.7...v2.50.8\n\n[2.50.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.6...v2.50.7\n\n[2.50.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.5...v2.50.6\n\n[2.50.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.4...v2.50.5\n\n[2.50.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.3...v2.50.4\n\n[2.50.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.2...v2.50.3\n\n[2.50.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.1...v2.50.2\n\n[2.50.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.0...v2.50.1\n\n[2.50.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.50...v2.50.0\n\n[2.49.50]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.49...v2.49.50\n\n[2.49.49]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.48...v2.49.49\n\n[2.49.48]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.47...v2.49.48\n\n[2.49.47]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.46...v2.49.47\n\n[2.49.46]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.45...v2.49.46\n\n[2.49.45]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.44...v2.49.45\n\n[2.49.44]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.43...v2.49.44\n\n[2.49.43]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.42...v2.49.43\n\n[2.49.42]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.41...v2.49.42\n\n[2.49.41]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.40...v2.49.41\n\n[2.49.40]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.39...v2.49.40\n\n[2.49.39]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.38...v2.49.39\n\n[2.49.38]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.37...v2.49.38\n\n[2.49.37]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.36...v2.49.37\n\n[2.49.36]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.35...v2.49.36\n\n[2.49.35]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.34...v2.49.35\n\n[2.49.34]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.33...v2.49.34\n\n[2.49.33]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.32...v2.49.33\n\n[2.49.32]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.31...v2.49.32\n\n[2.49.31]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.30...v2.49.31\n\n[2.49.30]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.29...v2.49.30\n\n[2.49.29]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.28...v2.49.29\n\n[2.49.28]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.27...v2.49.28\n\n[2.49.27]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.26...v2.49.27\n\n[2.49.26]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.25...v2.49.26\n\n[2.49.25]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.24...v2.49.25\n\n[2.49.24]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.23...v2.49.24\n\n[2.49.23]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.22...v2.49.23\n\n[2.49.22]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.21...v2.49.22\n\n[2.49.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.20...v2.49.21\n\n[2.49.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.19...v2.49.20\n\n[2.49.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.18...v2.49.19\n\n[2.49.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.17...v2.49.18\n\n[2.49.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.16...v2.49.17\n\n[2.49.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.15...v2.49.16\n\n[2.49.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.14...v2.49.15\n\n[2.49.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.13...v2.49.14\n\n[2.49.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.12...v2.49.13\n\n[2.49.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.11...v2.49.12\n\n[2.49.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.10...v2.49.11\n\n[2.49.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.9...v2.49.10\n\n[2.49.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.8...v2.49.9\n\n[2.49.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.7...v2.49.8\n\n[2.49.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.6...v2.49.7\n\n[2.49.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.5...v2.49.6\n\n[2.49.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.4...v2.49.5\n\n[2.49.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.3...v2.49.4\n\n[2.49.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.2...v2.49.3\n\n[2.49.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.1...v2.49.2\n\n[2.49.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.0...v2.49.1\n\n[2.49.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.22...v2.49.0\n\n[2.48.22]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.21...v2.48.22\n\n[2.48.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.20...v2.48.21\n\n[2.48.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.19...v2.48.20\n\n[2.48.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.18...v2.48.19\n\n[2.48.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.17...v2.48.18\n\n[2.48.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.16...v2.48.17\n\n[2.48.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.15...v2.48.16\n\n[2.48.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.14...v2.48.15\n\n[2.48.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.13...v2.48.14\n\n[2.48.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.12...v2.48.13\n\n[2.48.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.11...v2.48.12\n\n[2.48.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.10...v2.48.11\n\n[2.48.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.9...v2.48.10\n\n[2.48.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.8...v2.48.9\n\n[2.48.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.7...v2.48.8\n\n[2.48.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.6...v2.48.7\n\n[2.48.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.5...v2.48.6\n\n[2.48.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.4...v2.48.5\n\n[2.48.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.3...v2.48.4\n\n[2.48.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.2...v2.48.3\n\n[2.48.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.1...v2.48.2\n\n[2.48.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.0...v2.48.1\n\n[2.48.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.32...v2.48.0\n\n[2.47.32]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.31...v2.47.32\n\n[2.47.31]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.30...v2.47.31\n\n[2.47.30]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.29...v2.47.30\n\n[2.47.29]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.28...v2.47.29\n\n[2.47.28]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.27...v2.47.28\n\n[2.47.27]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.26...v2.47.27\n\n[2.47.26]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.25...v2.47.26\n\n[2.47.25]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.24...v2.47.25\n\n[2.47.24]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.23...v2.47.24\n\n[2.47.23]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.22...v2.47.23\n\n[2.47.22]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.21...v2.47.22\n\n[2.47.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.20...v2.47.21\n\n[2.47.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.19...v2.47.20\n\n[2.47.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.18...v2.47.19\n\n[2.47.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.17...v2.47.18\n\n[2.47.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.16...v2.47.17\n\n[2.47.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.15...v2.47.16\n\n[2.47.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.14...v2.47.15\n\n[2.47.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.13...v2.47.14\n\n[2.47.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.12...v2.47.13\n\n[2.47.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.11...v2.47.12\n\n[2.47.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.10...v2.47.11\n\n[2.47.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.9...v2.47.10\n\n[2.47.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.8...v2.47.9\n\n[2.47.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.7...v2.47.8\n\n[2.47.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.6...v2.47.7\n\n[2.47.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.5...v2.47.6\n\n[2.47.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.4...v2.47.5\n\n[2.47.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.3...v2.47.4\n\n[2.47.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.2...v2.47.3\n\n[2.47.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.1...v2.47.2\n\n[2.47.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.0...v2.47.1\n\n[2.47.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.20...v2.47.0\n\n[2.46.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.19...v2.46.20\n\n[2.46.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.18...v2.46.19\n\n[2.46.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.17...v2.46.18\n\n[2.46.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.16...v2.46.17\n\n[2.46.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.15...v2.46.16\n\n[2.46.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.14...v2.46.15\n\n[2.46.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.13...v2.46.14\n\n[2.46.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.12...v2.46.13\n\n[2.46.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.11...v2.46.12\n\n[2.46.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.10...v2.46.11\n\n[2.46.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.9...v2.46.10\n\n[2.46.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.8...v2.46.9\n\n[2.46.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.7...v2.46.8\n\n[2.46.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.6...v2.46.7\n\n[2.46.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.5...v2.46.6\n\n[2.46.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.4...v2.46.5\n\n[2.46.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.3...v2.46.4\n\n[2.46.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.2...v2.46.3\n\n[2.46.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.1...v2.46.2\n\n[2.46.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.0...v2.46.1\n\n[2.46.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.15...v2.46.0\n\n[2.45.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.14...v2.45.15\n\n[2.45.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.13...v2.45.14\n\n[2.45.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.12...v2.45.13\n\n[2.45.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.11...v2.45.12\n\n[2.45.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.10...v2.45.11\n\n[2.45.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.9...v2.45.10\n\n[2.45.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.8...v2.45.9\n\n[2.45.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.7...v2.45.8\n\n[2.45.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.6...v2.45.7\n\n[2.45.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.5...v2.45.6\n\n[2.45.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.4...v2.45.5\n\n[2.45.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.3...v2.45.4\n\n[2.45.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.2...v2.45.3\n\n[2.45.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.1...v2.45.2\n\n[2.45.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.0...v2.45.1\n\n[2.45.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.72...v2.45.0\n\n[2.44.72]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.71...v2.44.72\n\n[2.44.71]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.70...v2.44.71\n\n[2.44.70]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.69...v2.44.70\n\n[2.44.69]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.68...v2.44.69\n\n[2.44.68]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.67...v2.44.68\n\n[2.44.67]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.66...v2.44.67\n\n[2.44.66]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.65...v2.44.66\n\n[2.44.65]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.64...v2.44.65\n\n[2.44.64]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.63...v2.44.64\n\n[2.44.63]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.62...v2.44.63\n\n[2.44.62]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.61...v2.44.62\n\n[2.44.61]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.60...v2.44.61\n\n[2.44.60]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.59...v2.44.60\n\n[2.44.59]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.58...v2.44.59\n\n[2.44.58]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.57...v2.44.58\n\n[2.44.57]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.56...v2.44.57\n\n[2.44.56]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.55...v2.44.56\n\n[2.44.55]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.54...v2.44.55\n\n[2.44.54]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.53...v2.44.54\n\n[2.44.53]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.52...v2.44.53\n\n[2.44.52]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.51...v2.44.52\n\n[2.44.51]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.50...v2.44.51\n\n[2.44.50]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.49...v2.44.50\n\n[2.44.49]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.48...v2.44.49\n\n[2.44.48]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.47...v2.44.48\n\n[2.44.47]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.46...v2.44.47\n\n[2.44.46]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.45...v2.44.46\n\n[2.44.45]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.44...v2.44.45\n\n[2.44.44]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.43...v2.44.44\n\n[2.44.43]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.42...v2.44.43\n\n[2.44.42]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.41...v2.44.42\n\n[2.44.41]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.40...v2.44.41\n\n[2.44.40]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.39...v2.44.40\n\n[2.44.39]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.38...v2.44.39\n\n[2.44.38]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.37...v2.44.38\n\n[2.44.37]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.36...v2.44.37\n\n[2.44.36]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.35...v2.44.36\n\n[2.44.35]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.34...v2.44.35\n\n[2.44.34]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.33...v2.44.34\n\n[2.44.33]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.32...v2.44.33\n\n[2.44.32]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.31...v2.44.32\n\n[2.44.31]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.30...v2.44.31\n\n[2.44.30]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.29...v2.44.30\n\n[2.44.29]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.28...v2.44.29\n\n[2.44.28]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.27...v2.44.28\n\n[2.44.27]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.26...v2.44.27\n\n[2.44.26]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.25...v2.44.26\n\n[2.44.25]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.24...v2.44.25\n\n[2.44.24]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.23...v2.44.24\n\n[2.44.23]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.22...v2.44.23\n\n[2.44.22]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.21...v2.44.22\n\n[2.44.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.20...v2.44.21\n\n[2.44.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.19...v2.44.20\n\n[2.44.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.18...v2.44.19\n\n[2.44.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.17...v2.44.18\n\n[2.44.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.16...v2.44.17\n\n[2.44.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.15...v2.44.16\n\n[2.44.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.14...v2.44.15\n\n[2.44.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.13...v2.44.14\n\n[2.44.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.12...v2.44.13\n\n[2.44.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.11...v2.44.12\n\n[2.44.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.10...v2.44.11\n\n[2.44.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.9...v2.44.10\n\n[2.44.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.8...v2.44.9\n\n[2.44.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.7...v2.44.8\n\n[2.44.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.6...v2.44.7\n\n[2.44.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.5...v2.44.6\n\n[2.44.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.4...v2.44.5\n\n[2.44.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.3...v2.44.4\n\n[2.44.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.2...v2.44.3\n\n[2.44.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.1...v2.44.2\n\n[2.44.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.0...v2.44.1\n\n[2.44.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.43.7...v2.44.0\n\n[2.43.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.43.6...v2.43.7\n\n[2.43.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.43.5...v2.43.6\n\n[2.43.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.43.4...v2.43.5\n\n[2.43.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.43.3...v2.43.4\n\n[2.43.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.43.2...v2.43.3\n\n[2.43.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.43.1...v2.43.2\n\n[2.43.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.43.0...v2.43.1\n\n[2.43.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.42...v2.43.0\n\n[2.42.42]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.41...v2.42.42\n\n[2.42.41]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.40...v2.42.41\n\n[2.42.40]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.39...v2.42.40\n\n[2.42.39]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.38...v2.42.39\n\n[2.42.38]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.37...v2.42.38\n\n[2.42.37]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.36...v2.42.37\n\n[2.42.36]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.35...v2.42.36\n\n[2.42.35]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.34...v2.42.35\n\n[2.42.34]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.33...v2.42.34\n\n[2.42.33]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.32...v2.42.33\n\n[2.42.32]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.31...v2.42.32\n\n[2.42.31]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.30...v2.42.31\n\n[2.42.30]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.29...v2.42.30\n\n[2.42.29]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.28...v2.42.29\n\n[2.42.28]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.27...v2.42.28\n\n[2.42.27]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.26...v2.42.27\n\n[2.42.26]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.25...v2.42.26\n\n[2.42.25]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.24...v2.42.25\n\n[2.42.24]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.23...v2.42.24\n\n[2.42.23]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.22...v2.42.23\n\n[2.42.22]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.21...v2.42.22\n\n[2.42.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.20...v2.42.21\n\n[2.42.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.19...v2.42.20\n\n[2.42.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.18...v2.42.19\n\n[2.42.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.17...v2.42.18\n\n[2.42.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.16...v2.42.17\n\n[2.42.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.15...v2.42.16\n\n[2.42.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.14...v2.42.15\n\n[2.42.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.13...v2.42.14\n\n[2.42.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.12...v2.42.13\n\n[2.42.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.11...v2.42.12\n\n[2.42.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.10...v2.42.11\n\n[2.42.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.9...v2.42.10\n\n[2.42.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.8...v2.42.9\n\n[2.42.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.7...v2.42.8\n\n[2.42.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.6...v2.42.7\n\n[2.42.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.5...v2.42.6\n\n[2.42.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.4...v2.42.5\n\n[2.42.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.3...v2.42.4\n\n[2.42.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.2...v2.42.3\n\n[2.42.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.1...v2.42.2\n\n[2.42.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.0...v2.42.1\n\n[2.42.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.18...v2.42.0\n\n[2.41.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.17...v2.41.18\n\n[2.41.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.16...v2.41.17\n\n[2.41.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.15...v2.41.16\n\n[2.41.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.14...v2.41.15\n\n[2.41.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.13...v2.41.14\n\n[2.41.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.12...v2.41.13\n\n[2.41.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.11...v2.41.12\n\n[2.41.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.10...v2.41.11\n\n[2.41.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.9...v2.41.10\n\n[2.41.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.8...v2.41.9\n\n[2.41.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.7...v2.41.8\n\n[2.41.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.6...v2.41.7\n\n[2.41.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.5...v2.41.6\n\n[2.41.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.4...v2.41.5\n\n[2.41.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.3...v2.41.4\n\n[2.41.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.2...v2.41.3\n\n[2.41.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.1...v2.41.2\n\n[2.41.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.0...v2.41.1\n\n[2.41.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.40.2...v2.41.0\n\n[2.40.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.40.1...v2.40.2\n\n[2.40.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.40.0...v2.40.1\n\n[2.40.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.39.2...v2.40.0\n\n[2.39.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.39.1...v2.39.2\n\n[2.39.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.39.0...v2.39.1\n\n[2.39.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.38.7...v2.39.0\n\n[2.38.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.38.6...v2.38.7\n\n[2.38.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.38.5...v2.38.6\n\n[2.38.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.38.4...v2.38.5\n\n[2.38.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.38.3...v2.38.4\n\n[2.38.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.38.2...v2.38.3\n\n[2.38.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.38.1...v2.38.2\n\n[2.38.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.38.0...v2.38.1\n\n[2.38.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.37.0...v2.38.0\n\n[2.37.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.36.0...v2.37.0\n\n[2.36.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.35.0...v2.36.0\n\n[2.35.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.34.3...v2.35.0\n\n[2.34.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.34.2...v2.34.3\n\n[2.34.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.34.1...v2.34.2\n\n[2.34.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.34.0...v2.34.1\n\n[2.34.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.36...v2.34.0\n\n[2.33.36]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.35...v2.33.36\n\n[2.33.35]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.34...v2.33.35\n\n[2.33.34]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.33...v2.33.34\n\n[2.33.33]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.32...v2.33.33\n\n[2.33.32]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.31...v2.33.32\n\n[2.33.31]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.30...v2.33.31\n\n[2.33.30]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.29...v2.33.30\n\n[2.33.29]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.28...v2.33.29\n\n[2.33.28]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.27...v2.33.28\n\n[2.33.27]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.26...v2.33.27\n\n[2.33.26]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.25...v2.33.26\n\n[2.33.25]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.24...v2.33.25\n\n[2.33.24]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.23...v2.33.24\n\n[2.33.23]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.22...v2.33.23\n\n[2.33.22]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.21...v2.33.22\n\n[2.33.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.20...v2.33.21\n\n[2.33.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.19...v2.33.20\n\n[2.33.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.18...v2.33.19\n\n[2.33.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.17...v2.33.18\n\n[2.33.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.16...v2.33.17\n\n[2.33.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.15...v2.33.16\n\n[2.33.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.14...v2.33.15\n\n[2.33.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.13...v2.33.14\n\n[2.33.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.12...v2.33.13\n\n[2.33.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.11...v2.33.12\n\n[2.33.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.10...v2.33.11\n\n[2.33.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.9...v2.33.10\n\n[2.33.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.8...v2.33.9\n\n[2.33.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.7...v2.33.8\n\n[2.33.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.6...v2.33.7\n\n[2.33.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.5...v2.33.6\n\n[2.33.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.4...v2.33.5\n\n[2.33.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.3...v2.33.4\n\n[2.33.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.2...v2.33.3\n\n[2.33.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.1...v2.33.2\n\n[2.33.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.0...v2.33.1\n\n[2.33.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.20...v2.33.0\n\n[2.32.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.19...v2.32.20\n\n[2.32.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.18...v2.32.19\n\n[2.32.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.17...v2.32.18\n\n[2.32.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.16...v2.32.17\n\n[2.32.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.15...v2.32.16\n\n[2.32.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.14...v2.32.15\n\n[2.32.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.13...v2.32.14\n\n[2.32.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.12...v2.32.13\n\n[2.32.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.11...v2.32.12\n\n[2.32.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.10...v2.32.11\n\n[2.32.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.9...v2.32.10\n\n[2.32.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.8...v2.32.9\n\n[2.32.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.7...v2.32.8\n\n[2.32.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.6...v2.32.7\n\n[2.32.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.5...v2.32.6\n\n[2.32.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.4...v2.32.5\n\n[2.32.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.3...v2.32.4\n\n[2.32.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.2...v2.32.3\n\n[2.32.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.1...v2.32.2\n\n[2.32.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.0...v2.32.1\n\n[2.32.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.31.3...v2.32.0\n\n[2.31.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.31.2...v2.31.3\n\n[2.31.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.31.1...v2.31.2\n\n[2.31.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.31.0...v2.31.1\n\n[2.31.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.30.0...v2.31.0\n\n[2.30.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.29.8...v2.30.0\n\n[2.29.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.29.7...v2.29.8\n\n[2.29.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.29.6...v2.29.7\n\n[2.29.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.29.5...v2.29.6\n\n[2.29.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.29.4...v2.29.5\n\n[2.29.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.29.3...v2.29.4\n\n[2.29.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.29.2...v2.29.3\n\n[2.29.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.29.1...v2.29.2\n\n[2.29.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.29.0...v2.29.1\n\n[2.29.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.16...v2.29.0\n\n[2.28.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.15...v2.28.16\n\n[2.28.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.14...v2.28.15\n\n[2.28.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.13...v2.28.14\n\n[2.28.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.12...v2.28.13\n\n[2.28.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.11...v2.28.12\n\n[2.28.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.10...v2.28.11\n\n[2.28.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.9...v2.28.10\n\n[2.28.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.8...v2.28.9\n\n[2.28.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.7...v2.28.8\n\n[2.28.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.6...v2.28.7\n\n[2.28.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.5...v2.28.6\n\n[2.28.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.4...v2.28.5\n\n[2.28.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.3...v2.28.4\n\n[2.28.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.2...v2.28.3\n\n[2.28.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.1...v2.28.2\n\n[2.28.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.0...v2.28.1\n\n[2.28.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.15...v2.28.0\n\n[2.27.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.14...v2.27.15\n\n[2.27.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.13...v2.27.14\n\n[2.27.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.12...v2.27.13\n\n[2.27.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.11...v2.27.12\n\n[2.27.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.10...v2.27.11\n\n[2.27.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.9...v2.27.10\n\n[2.27.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.8...v2.27.9\n\n[2.27.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.7...v2.27.8\n\n[2.27.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.6...v2.27.7\n\n[2.27.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.5...v2.27.6\n\n[2.27.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.4...v2.27.5\n\n[2.27.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.3...v2.27.4\n\n[2.27.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.2...v2.27.3\n\n[2.27.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.1...v2.27.2\n\n[2.27.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.0...v2.27.1\n\n[2.27.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.20...v2.27.0\n\n[2.26.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.19...v2.26.20\n\n[2.26.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.18...v2.26.19\n\n[2.26.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.17...v2.26.18\n\n[2.26.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.16...v2.26.17\n\n[2.26.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.15...v2.26.16\n\n[2.26.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.14...v2.26.15\n\n[2.26.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.13...v2.26.14\n\n[2.26.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.12...v2.26.13\n\n[2.26.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.11...v2.26.12\n\n[2.26.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.10...v2.26.11\n\n[2.26.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.9...v2.26.10\n\n[2.26.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.8...v2.26.9\n\n[2.26.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.7...v2.26.8\n\n[2.26.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.6...v2.26.7\n\n[2.26.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.5...v2.26.6\n\n[2.26.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.4...v2.26.5\n\n[2.26.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.3...v2.26.4\n\n[2.26.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.2...v2.26.3\n\n[2.26.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.1...v2.26.2\n\n[2.26.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.0...v2.26.1\n\n[2.26.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.25.11...v2.26.0\n\n[2.25.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.25.10...v2.25.11\n\n[2.25.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.25.9...v2.25.10\n\n[2.25.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.25.8...v2.25.9\n\n[2.25.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.25.7...v2.25.8\n\n[2.25.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.25.6...v2.25.7\n\n[2.25.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.25.5...v2.25.6\n\n[2.25.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.25.4...v2.25.5\n\n[2.25.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.25.3...v2.25.4\n\n[2.25.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.25.2...v2.25.3\n\n[2.25.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.25.1...v2.25.2\n\n[2.25.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.25.0...v2.25.1\n\n[2.25.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.24.4...v2.25.0\n\n[2.24.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.24.3...v2.24.4\n\n[2.24.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.24.2...v2.24.3\n\n[2.24.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.24.1...v2.24.2\n\n[2.24.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.24.0...v2.24.1\n\n[2.24.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.23.9...v2.24.0\n\n[2.23.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.23.8...v2.23.9\n\n[2.23.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.23.7...v2.23.8\n\n[2.23.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.23.6...v2.23.7\n\n[2.23.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.23.5...v2.23.6\n\n[2.23.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.23.4...v2.23.5\n\n[2.23.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.23.3...v2.23.4\n\n[2.23.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.23.2...v2.23.3\n\n[2.23.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.23.1...v2.23.2\n\n[2.23.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.23.0...v2.23.1\n\n[2.23.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.22.10...v2.23.0\n\n[2.22.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.22.9...v2.22.10\n\n[2.22.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.22.8...v2.22.9\n\n[2.22.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.22.7...v2.22.8\n\n[2.22.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.22.6...v2.22.7\n\n[2.22.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.22.5...v2.22.6\n\n[2.22.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.22.4...v2.22.5\n\n[2.22.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.22.3...v2.22.4\n\n[2.22.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.22.2...v2.22.3\n\n[2.22.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.22.1...v2.22.2\n\n[2.22.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.22.0...v2.22.1\n\n[2.22.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.21.27...v2.22.0\n\n[2.21.27]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.21.26...v2.21.27\n\n[2.21.26]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.21.25...v2.21.26\n\n[2.21.25]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.21.24...v2.21.25\n\n[2.21.24]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.21.23...v2.21.24\n\n[2.21.23]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.21.22...v2.21.23\n\n[2.21\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - \"before 8am on Monday\" (UTC),\nAutomerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0MS4xNDMuMSIsInVwZGF0ZWRJblZlciI6IjQxLjE0My4xIiwidGFyZ2V0QnJhbmNoIjoibWFpbiIsImxhYmVscyI6WyJkZXBlbmRlbmNpZXMiXX0=-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2025-10-16T16:01:16Z",
          "tree_id": "f052a304c90f254b4a9eae454eefc4817659da42",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/ac76786947bcac5a29e688218a68038ec5fd0f83"
        },
        "date": 1760630995230,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": -0.03333333333333333,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": -1,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 0,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 22.95,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 23.05,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": -0.03333333333333333,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": -1,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 0,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-avg",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 22.13,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 22.24,
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
          "id": "f6ca6eef6de794993da517fc9eb87bf4cf1a3e66",
          "message": "Upgrade various Go dependencies (including Collector v0.137.0/v1.43.0) (#1298)\n\nFollowing up closed Renovate PR\nhttps://github.com/open-telemetry/otel-arrow/pull/1272\n\nTargets non-breaking upgrades mentioned in that PR",
          "timestamp": "2025-10-16T16:53:36Z",
          "tree_id": "684c533661403a615afef61b393abedc5a691391",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/f6ca6eef6de794993da517fc9eb87bf4cf1a3e66"
        },
        "date": 1760634131296,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "pipeline-perf-collector-config-throughput",
            "value": -0.03333333333333333,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-logs-sent",
            "value": -1,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-logs-received",
            "value": 0,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-avg",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-cpu-max",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-memory-avg",
            "value": 21.5,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-memory-max",
            "value": 21.61,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-throughput",
            "value": -0.03333333333333333,
            "unit": "logs/sec"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-sent",
            "value": -1,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-logs-received",
            "value": 0,
            "unit": "count"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-loss-percentage",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-avg",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-cpu-max",
            "value": 0,
            "unit": "percent"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-avg",
            "value": 22.48,
            "unit": "MiB"
          },
          {
            "name": "pipeline-perf-collector-config-with-batch-processor-memory-max",
            "value": 22.48,
            "unit": "MiB"
          }
        ]
      }
    ]
  }
}