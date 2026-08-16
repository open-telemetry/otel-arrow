window.BENCHMARK_DATA = {
  "lastUpdate": 1786902485308,
  "repoUrl": "https://github.com/open-telemetry/otel-arrow",
  "entries": {
    "Benchmark": [
      {
        "commit": {
          "author": {
            "name": "Lalit Kumar Bhasin",
            "username": "lalitb",
            "email": "lalit_fin@yahoo.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "4eb82c9408649b996a5049442495b49195efe0c8",
          "message": "feat(engine): add NUMA-aware core placement planning (#3471)\n\n# Change Summary\n\n**_Note:_** The implementation is based on the design proposal #3317,\nand so subject to the design approval.\n\nThis PR moves pipeline core placement into the controller and makes it\ntopology-aware.\n\nToday, `core_count` pipelines each pick their own cores, starting from\nthe lowest. Two pipelines can silently land on the same cores and\ncompete, and nothing in the engine knows which NUMA node a core belongs\nto.\n\nThis change resolves placement explicitly, before any pipeline launches:\n\n- The engine discovers the process-visible CPU set and the CPU-to-NUMA\nmapping.\n- The controller plans `core_count` placement globally, accounting for\nother pipelines' reserved cores.\n- `core_count` pipelines receive exclusive cores instead of silently\noverlapping.\n- Explicit `core_set` is still honored, but now fails if a requested\ncore is hidden by process affinity or cgroup limits.\n- Startup, live rollout, rollback, and full-config reconcile all share\none placement model.\n\nThe default policy is deterministic NUMA packing: it keeps a pipeline on\na single NUMA node when possible, using stable lowest-node then\nlowest-core ordering. When topology is incomplete, it falls back to\ndeterministic visible-core ordering. The policy sits behind a small\nstrategy interface, so balancing or hardware-aware strategies can be\nadded later without touching placement call sites.\n\nThis PR also adds listener-group metadata as groundwork for future\nsocket-placement work. It does **not** bind sockets, enable\n`SO_REUSEPORT`, or attach eBPF selectors - there is no production\nruntime consumer yet.\n\nThe per-record data path is unchanged. Topology discovery and placement\nplanning run only at startup and during live control operations.\n\n## Breaking Behavior Changes\n\nThese configs now fail loudly instead of doing something surprising:\n\n- `core_count` no longer clamps to the available core count. Requesting\nmore cores than are visible is now a validation error.\n- Multiple `core_count` pipelines no longer overlap on the same first\ncores. Each gets an exclusive set, and startup or live update fails when\nthere aren't enough unreserved cores.\n- `core_count: 0` means \"all unreserved visible cores\" and can now fail\nif none remain.\n- Explicit `core_set` fails if any requested core is hidden by process\naffinity or cgroup CPU limits.\n- Full-config reconcile rejects placement transitions that need another\nlive pipeline to vacate cores first. Stage the shrink or delete first,\nthen apply the growth.\n\nExplicit `core_set`-to-`core_set` overlap is still allowed, as\ndeliberate operator intent.\n\n  * Closes #1837\n\nRelated context, not closed by this PR: #2155 (placement abstraction /\nbalancing), #2974 (socket + eBPF placement).\n\n  ## How are these changes tested?\n\n  New and updated unit coverage:\n\n- Linux topology discovery from sysfs, affinity, and cgroup v2 cpuset\nlimits\n- complete, partial, and unknown topology states, including disjoint\naffinity/cgroup visibility\n  - cpulist parse errors, oversized ranges, and duplicate CPU mappings\n  - NUMA-packing placement and deterministic fallback ordering\n  - strategy injection via the placement interface\n  - startup reservation conflicts across pipelines\n  - `core_count: 0` and omitted-count behavior\n  - explicit `core_set` hidden-core rejection\n  - live rollout, rollback, and reconcile placement handling\n  - conservative vacate-before-claim reconcile rejection\n\n  ## Are there any user-facing changes?\n\n  Yes — see **Breaking Behavior Changes** above.\n\n  ### Changelog\n\n  * [x] Added a `.chloggen/*.yaml` entry\n  * [ ] This PR is a `chore` (indicated in title)\n  * [ ] This is a documentation-only PR.",
          "timestamp": "2026-08-13T21:38:52Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/4eb82c9408649b996a5049442495b49195efe0c8"
        },
        "date": 1786673457741,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "log_rows_written_rate",
            "value": 99232.73184562218,
            "unit": "rows/sec",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - ClickHouse rows written"
          },
          {
            "name": "logs_produced_rate",
            "value": 100873.45642026469,
            "unit": "rows/sec",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - logs produced"
          },
          {
            "name": "logs_produced_rate",
            "value": 100921.36157045748,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - logs_produced"
          },
          {
            "name": "log_rows_written_rate",
            "value": 100073.32035264392,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - ClickHouse rows written"
          },
          {
            "name": "logs_produced_rate",
            "value": 100262.11405522404,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - logs_produced"
          },
          {
            "name": "log_rows_written_rate",
            "value": 100080.48825990778,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - ClickHouse rows written"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Drew Relmas",
            "username": "drewrelmas",
            "email": "drewrelmas@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "bf9ff5d0d28abf0aec68c40dd1d4691c43aa52dc",
          "message": "chore(release): Avoid edge case race conditions during Release prep and push (#3755)\n\n# Change Summary\n\nProtect the release window from changelog gaps.\n\nThe release process previously rendered the changelog at Prepare Release\ntime but created tags from the current `main` tip later. A pull request\nmerged during that window could therefore be included in the release\ntags without appearing in that release's changelog.\n\nThis change closes the gap at three points:\n\n1. **Before and after opening the release PR:** Prepare Release records\nits starting `main` commit and verifies that the merge queue is empty\nand `main` has not moved. If the first check fails, no PR is opened. If\nthe second check fails, the release PR remains open to freeze merges and\nPrepare Release must be rerun to update it from the latest `main`.\n2. **While the release PR is open:** The required `changelog` check\nrejects every other pull request and merge-group entry. The release PR\nis allowed only when no other PR remains queued and its branch contains\nthe latest `main`.\n3. **When creating the release:** Push Release resolves the uniquely\nmerged `otelbot/release-v<VERSION>` PR and creates all release tags at\nthat PR's merge commit rather than the current `main` tip.\n\n```mermaid\nsequenceDiagram\n    participant MQ as Merge queue\n    participant Prep as Prepare Release\n    participant Main as main\n    participant RP as Release PR\n    participant Push as Push Release\n\n    Prep->>Main: Record starting commit\n    Prep->>MQ: Require empty queue\n    Prep->>Main: Require unchanged commit\n    Prep->>RP: Open otelbot/release-vX.Y.Z\n    Prep->>MQ: Recheck empty queue\n    Prep->>Main: Recheck unchanged commit\n\n    Note over RP,MQ: Required changelog check freezes unrelated merges\n    RP->>MQ: Require no other queued PRs\n    RP->>Main: Merge prepared changelog and versions\n\n    Push->>RP: Resolve merged release PR\n    Push->>Main: Tag the release PR merge commit\n    Note over Main,Push: Later main commits remain pending for the next release\n```\n\n## What issue does this PR close?\n\n<!--We highly recommend correlation of every PR to an issue-->\n\nN/A\n\n## How are these changes tested?\n\nN/A\n\n## Are there any user-facing changes?\n\nNo\n\n### Changelog\n\n* [x] This PR is a `chore` (indicated in title)\n\n---------\n\nCo-authored-by: Copilot Autofix powered by AI <175728472+Copilot@users.noreply.github.com>",
          "timestamp": "2026-08-14T16:51:26Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/bf9ff5d0d28abf0aec68c40dd1d4691c43aa52dc"
        },
        "date": 1786730481557,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "log_rows_written_rate",
            "value": 98961.9824250295,
            "unit": "rows/sec",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - ClickHouse rows written"
          },
          {
            "name": "logs_produced_rate",
            "value": 100998.28557093549,
            "unit": "rows/sec",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - logs produced"
          },
          {
            "name": "logs_produced_rate",
            "value": 98497.96044688727,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - logs_produced"
          },
          {
            "name": "log_rows_written_rate",
            "value": 99940.3689132151,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - ClickHouse rows written"
          },
          {
            "name": "logs_produced_rate",
            "value": 100959.68915184772,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - logs_produced"
          },
          {
            "name": "log_rows_written_rate",
            "value": 99937.03966501105,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - ClickHouse rows written"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "c1ly",
            "username": "c1ly",
            "email": "129437996+c1ly@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "5abeaf8efbd37b6c8519fc8edcd519281e988bbd",
          "message": "[otap-dataflow] Kafka Exporter enhancement + integration tests (#3683)\n\n# Change Summary\n\n- Updated to immediately send a permanent nack for unretryable errors\nthat occur when attempting to send data\n- Add live reconfiguration support\n- Updated config to allow control of dynamic topics via allowlist and\nregex\n- Updated config to allow user to directly update config setting for\nautomatic topic creation\n- Added and reorganized test under various scenario types\n  - Security\n  - Shutdown & Live Reconfiguration\n  - Retry expectations\n  - Delivery semantics\n  - Kafka Integration\n  - Telemetry/Observability\n  - Configuration\n  \n## What issue does this PR close?\n\n* Completes part of #3509 \n\n## How are these changes tested?\n\nintegration tests and unit tests\n\n## Are there any user-facing changes?\n\nyes, allow_auto_create_topics is now exposed directly in the Kafka\nExporter config and overwrites any changes set in the producer_config\nhash map (if configured)\n\n### Changelog\n\n* [ x ] Added a `.chloggen/*.yaml` entry\n\n---------\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2026-08-15T01:02:30Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/5abeaf8efbd37b6c8519fc8edcd519281e988bbd"
        },
        "date": 1786759111975,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "log_rows_written_rate",
            "value": 99357.59717782235,
            "unit": "rows/sec",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - ClickHouse rows written"
          },
          {
            "name": "logs_produced_rate",
            "value": 100819.72588859516,
            "unit": "rows/sec",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - logs produced"
          },
          {
            "name": "logs_produced_rate",
            "value": 99937.45609103888,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - logs_produced"
          },
          {
            "name": "log_rows_written_rate",
            "value": 99948.85950022239,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - ClickHouse rows written"
          },
          {
            "name": "logs_produced_rate",
            "value": 100846.38861266784,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - logs_produced"
          },
          {
            "name": "log_rows_written_rate",
            "value": 100083.48908411876,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - ClickHouse rows written"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "c1ly",
            "username": "c1ly",
            "email": "129437996+c1ly@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "5abeaf8efbd37b6c8519fc8edcd519281e988bbd",
          "message": "[otap-dataflow] Kafka Exporter enhancement + integration tests (#3683)\n\n# Change Summary\n\n- Updated to immediately send a permanent nack for unretryable errors\nthat occur when attempting to send data\n- Add live reconfiguration support\n- Updated config to allow control of dynamic topics via allowlist and\nregex\n- Updated config to allow user to directly update config setting for\nautomatic topic creation\n- Added and reorganized test under various scenario types\n  - Security\n  - Shutdown & Live Reconfiguration\n  - Retry expectations\n  - Delivery semantics\n  - Kafka Integration\n  - Telemetry/Observability\n  - Configuration\n  \n## What issue does this PR close?\n\n* Completes part of #3509 \n\n## How are these changes tested?\n\nintegration tests and unit tests\n\n## Are there any user-facing changes?\n\nyes, allow_auto_create_topics is now exposed directly in the Kafka\nExporter config and overwrites any changes set in the producer_config\nhash map (if configured)\n\n### Changelog\n\n* [ x ] Added a `.chloggen/*.yaml` entry\n\n---------\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2026-08-15T01:02:30Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/5abeaf8efbd37b6c8519fc8edcd519281e988bbd"
        },
        "date": 1786816072612,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "log_rows_written_rate",
            "value": 99258.39724940491,
            "unit": "rows/sec",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - ClickHouse rows written"
          },
          {
            "name": "logs_produced_rate",
            "value": 100952.0704208791,
            "unit": "rows/sec",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - logs produced"
          },
          {
            "name": "logs_produced_rate",
            "value": 100951.58308234529,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - logs_produced"
          },
          {
            "name": "log_rows_written_rate",
            "value": 99941.36773093118,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - ClickHouse rows written"
          },
          {
            "name": "logs_produced_rate",
            "value": 100304.06676318929,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - logs_produced"
          },
          {
            "name": "log_rows_written_rate",
            "value": 99941.20125992541,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - ClickHouse rows written"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "c1ly",
            "username": "c1ly",
            "email": "129437996+c1ly@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "5abeaf8efbd37b6c8519fc8edcd519281e988bbd",
          "message": "[otap-dataflow] Kafka Exporter enhancement + integration tests (#3683)\n\n# Change Summary\n\n- Updated to immediately send a permanent nack for unretryable errors\nthat occur when attempting to send data\n- Add live reconfiguration support\n- Updated config to allow control of dynamic topics via allowlist and\nregex\n- Updated config to allow user to directly update config setting for\nautomatic topic creation\n- Added and reorganized test under various scenario types\n  - Security\n  - Shutdown & Live Reconfiguration\n  - Retry expectations\n  - Delivery semantics\n  - Kafka Integration\n  - Telemetry/Observability\n  - Configuration\n  \n## What issue does this PR close?\n\n* Completes part of #3509 \n\n## How are these changes tested?\n\nintegration tests and unit tests\n\n## Are there any user-facing changes?\n\nyes, allow_auto_create_topics is now exposed directly in the Kafka\nExporter config and overwrites any changes set in the producer_config\nhash map (if configured)\n\n### Changelog\n\n* [ x ] Added a `.chloggen/*.yaml` entry\n\n---------\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2026-08-15T01:02:30Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/5abeaf8efbd37b6c8519fc8edcd519281e988bbd"
        },
        "date": 1786845491344,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "log_rows_written_rate",
            "value": 99331.95701656873,
            "unit": "rows/sec",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - ClickHouse rows written"
          },
          {
            "name": "logs_produced_rate",
            "value": 99170.01421492897,
            "unit": "rows/sec",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - logs produced"
          },
          {
            "name": "logs_produced_rate",
            "value": 100905.7451039145,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - logs_produced"
          },
          {
            "name": "log_rows_written_rate",
            "value": 100075.48728608881,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - ClickHouse rows written"
          },
          {
            "name": "logs_produced_rate",
            "value": 100416.5653565641,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - logs_produced"
          },
          {
            "name": "log_rows_written_rate",
            "value": 100083.98923898286,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - ClickHouse rows written"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "c1ly",
            "username": "c1ly",
            "email": "129437996+c1ly@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "5abeaf8efbd37b6c8519fc8edcd519281e988bbd",
          "message": "[otap-dataflow] Kafka Exporter enhancement + integration tests (#3683)\n\n# Change Summary\n\n- Updated to immediately send a permanent nack for unretryable errors\nthat occur when attempting to send data\n- Add live reconfiguration support\n- Updated config to allow control of dynamic topics via allowlist and\nregex\n- Updated config to allow user to directly update config setting for\nautomatic topic creation\n- Added and reorganized test under various scenario types\n  - Security\n  - Shutdown & Live Reconfiguration\n  - Retry expectations\n  - Delivery semantics\n  - Kafka Integration\n  - Telemetry/Observability\n  - Configuration\n  \n## What issue does this PR close?\n\n* Completes part of #3509 \n\n## How are these changes tested?\n\nintegration tests and unit tests\n\n## Are there any user-facing changes?\n\nyes, allow_auto_create_topics is now exposed directly in the Kafka\nExporter config and overwrites any changes set in the producer_config\nhash map (if configured)\n\n### Changelog\n\n* [ x ] Added a `.chloggen/*.yaml` entry\n\n---------\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2026-08-15T01:02:30Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/5abeaf8efbd37b6c8519fc8edcd519281e988bbd"
        },
        "date": 1786902484289,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "log_rows_written_rate",
            "value": 98977.5340059982,
            "unit": "rows/sec",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - ClickHouse rows written"
          },
          {
            "name": "logs_produced_rate",
            "value": 100482.02076905464,
            "unit": "rows/sec",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - logs produced"
          },
          {
            "name": "logs_produced_rate",
            "value": 100268.0794563689,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - logs_produced"
          },
          {
            "name": "log_rows_written_rate",
            "value": 99933.71063860973,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - ClickHouse rows written"
          },
          {
            "name": "logs_produced_rate",
            "value": 100897.31004744493,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - logs_produced"
          },
          {
            "name": "log_rows_written_rate",
            "value": 99939.37011546329,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - ClickHouse rows written"
          }
        ]
      }
    ]
  }
}