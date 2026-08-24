window.BENCHMARK_DATA = {
  "lastUpdate": 1787535294009,
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
        "date": 1786931868194,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "log_rows_written_rate",
            "value": 99115.45036008696,
            "unit": "rows/sec",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - ClickHouse rows written"
          },
          {
            "name": "logs_produced_rate",
            "value": 101007.37047358367,
            "unit": "rows/sec",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - logs produced"
          },
          {
            "name": "logs_produced_rate",
            "value": 100346.23362402615,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - logs_produced"
          },
          {
            "name": "log_rows_written_rate",
            "value": 100073.48703650034,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - ClickHouse rows written"
          },
          {
            "name": "logs_produced_rate",
            "value": 99287.80186583992,
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
            "name": "renovate[bot]",
            "username": "renovate[bot]",
            "email": "29139614+renovate[bot]@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "57058ad9b922bd8c170c04c3ab6dd0258000c40f",
          "message": "chore(deps): update geneva-uploader digest to 7819998 (#3779)\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n|---|---|---|---|\n| geneva-uploader | workspace.dependencies | digest | `f101f2a` →\n`7819998` |\n\n---\n\n### Configuration\n\n📅 **Schedule**: (UTC)\n\n- Branch creation\n  - \"before 8am on Monday\"\n- Automerge\n  - At any time (no schedule defined)\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0NC4yOS41IiwidXBkYXRlZEluVmVyIjoiNDQuMjkuNSIsInRhcmdldEJyYW5jaCI6Im1haW4iLCJsYWJlbHMiOlsiZGVwZW5kZW5jaWVzIl19-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2026-08-17T06:16:31Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/57058ad9b922bd8c170c04c3ab6dd0258000c40f"
        },
        "date": 1786988987644,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "log_rows_written_rate",
            "value": 99269.88378052208,
            "unit": "rows/sec",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - ClickHouse rows written"
          },
          {
            "name": "logs_produced_rate",
            "value": 99414.4086930385,
            "unit": "rows/sec",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - logs produced"
          },
          {
            "name": "logs_produced_rate",
            "value": 100212.68869284944,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - logs_produced"
          },
          {
            "name": "log_rows_written_rate",
            "value": 99949.35899144433,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - ClickHouse rows written"
          },
          {
            "name": "logs_produced_rate",
            "value": 99131.62044646461,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - logs_produced"
          },
          {
            "name": "log_rows_written_rate",
            "value": 100106.07110495176,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - ClickHouse rows written"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Laurent Quérel",
            "username": "lquerel",
            "email": "l.querel@f5.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "55aebb067841cf02a78ea7bb4bc24e838c8945b0",
          "message": "feat(file-exporter): add multi-signal file exporter (#3776)\n\n# Change Summary\n\nAdds an experimental multi-signal file exporter for OTAP Dataflow. The\nexporter writes logs, metrics, and traces as bounded OTLP JSON lines and\nsupports multi-core path templates, configurable open and durability\nmodes, partial-tail recovery, and exporter telemetry.\n\n## What issue does this PR close?\n\n- Closes #3773\n\n## How are these changes tested?\n\n- Unit and integration tests covering all three OTel signals\n- Configuration and path-collision validation tests\n- File open mode, tail recovery, path lease, and frame-size tests\n- cargo xtask check\n\n## Are there any user-facing changes?\n\nYes. Users can configure the experimental `exporter:file` component to\ncapture logs, metrics, and traces as OTLP JSONL files.\n\n### Changelog\n\n- [x] Added a .chloggen/*.yaml entry\n- [ ] This PR is a chore (indicated in title)\n- [ ] This is a documentation-only PR.",
          "timestamp": "2026-08-18T00:57:14Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/55aebb067841cf02a78ea7bb4bc24e838c8945b0"
        },
        "date": 1787019108015,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "log_rows_written_rate",
            "value": 99206.37251424263,
            "unit": "rows/sec",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - ClickHouse rows written"
          },
          {
            "name": "logs_produced_rate",
            "value": 101057.12319575771,
            "unit": "rows/sec",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - logs produced"
          },
          {
            "name": "logs_produced_rate",
            "value": 100257.64515522773,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - logs_produced"
          },
          {
            "name": "log_rows_written_rate",
            "value": 100085.48973356921,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - ClickHouse rows written"
          },
          {
            "name": "logs_produced_rate",
            "value": 100882.32183084868,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - logs_produced"
          },
          {
            "name": "log_rows_written_rate",
            "value": 99942.03362050011,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - ClickHouse rows written"
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
          "id": "46f0898750a81543bad4dd245339f02ff8713b5f",
          "message": "chore(repo): Split build/test chains and avoid full CI repeat after merge (#3807)\n\n# Chore Summary\n\n- Split the required Linux and Windows Rust build/test chains so each\ntest matrix starts as soon as its matching build completes\n- Make Rust-CI caches restore-only to prevent ephemeral pull-request and\nmerge-queue refs from consuming the repository cache budget\n- Replace the full Rust-CI run on `main` with a `Post-Merge Actions`\nworkflow that warms the required Linux and Windows build caches\n\n## Expected impact\n\n| Area | Before | After | Potential gain |\n| --- | --- | --- | --- |\n| Required test chain | Linux tests wait for the slower Windows build |\nLinux and Windows tests start after their matching builds |\nApproximately 7-9 minutes removed from the Linux test path |\n| End-to-end required status | The delayed Linux test path can determine\nworkflow completion | Other required jobs may become the critical path |\nApproximately 2 minutes in a sampled PR run and 8-9 minutes in a sampled\nmerge-queue run |\n| Rust-CI cache writes | Large job-specific caches are saved under\nephemeral PR and merge-queue refs | Rust-CI restores caches but never\nsaves them | Reduces churn within GitHub's 10 GB repository cache limit\n|\n| Post-merge Rust-CI | Approximately 6.4 runner-hours across 57 jobs |\nApproximately 0.8 runner-hours across two cache-warming jobs |\nApproximately 85% less compute |\n| Shared cache priority | The full suite competes to publish many\njob-specific caches | `main` publishes the required Linux and Windows\nbuild caches | Prioritizes the merge-critical build configurations |\n\n## Tradeoff\n\nThe post-merge workflow intentionally does not warm Rust-CI's coverage,\nclippy, smoke-test, ARM, or macOS caches. Those configurations can still\nrestore an existing compatible cache, but only the required Linux and\nWindows build caches are guaranteed to be refreshed after each merge.\n\nThe current cache inventory supports prioritizing these builds:\n\n| Observation | Current state |\n| --- | --- |\n| Repository cache limit | 10 GB |\n| Recent Rust cache size | Commonly 1-2 GB per entry |\n| Dominant cache refs | `refs/pull/*/merge` entries from recent PR runs\n|\n| Shared `main` Rust caches | None were present in the inspected cache\ninventory |\n\nThis means a small number of ephemeral PR entries can consume the\nrepository budget and evict caches that would otherwise be reusable by\nevery PR and merge-queue run. The restore-only Rust-CI policy stops\nadding those ephemeral entries, while the post-merge workflow\nrepopulates the two critical shared caches.",
          "timestamp": "2026-08-18T20:06:27Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/46f0898750a81543bad4dd245339f02ff8713b5f"
        },
        "date": 1787090394666,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "log_rows_written_rate",
            "value": 98951.87151675747,
            "unit": "rows/sec",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - ClickHouse rows written"
          },
          {
            "name": "logs_produced_rate",
            "value": 99321.30720644261,
            "unit": "rows/sec",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - logs produced"
          },
          {
            "name": "logs_produced_rate",
            "value": 99116.1860091624,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - logs_produced"
          },
          {
            "name": "log_rows_written_rate",
            "value": 99941.70067460649,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - ClickHouse rows written"
          },
          {
            "name": "logs_produced_rate",
            "value": 100452.84981805058,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - logs_produced"
          },
          {
            "name": "log_rows_written_rate",
            "value": 99947.0280751202,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - ClickHouse rows written"
          }
        ]
      },
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
          "id": "2e9fff0f0b8808f44b663fdf27a09b2034a5096b",
          "message": "  feat(geneva-exporter): support account group routing (#3799)\n\n## Summary\n\n- add required `account_routing.default_group` configuration to the\nGeneva exporter\n- support optional destination event/table overrides through\n`account_routing.events`\n  - pass account routing into `geneva-uploader`\n- preserve the complete logical-group-to-primary-moniker map from\nagent-fed credentials\n- update `geneva-uploader` to the revision containing multi-moniker\nrouting\n  - update Geneva examples, documentation, tests, and changelog\n\n  This integrates the account-group routing added by\n  https://github.com/open-telemetry/opentelemetry-rust-contrib/pull/747.\n\n  ## Routing model\n\n  YAML config selects logical GCS account groups:\n\n  ```yaml\n  account_routing:\n    default_group: \"diagnostics\"\n    events:\n      AuditLogs: \"audit\"\n      SecurityEvents: \"security\"\n```\n\n  The events keys are final destination event/table names after\n  event_name_mapping has run. Events without an exact override use\n  default_group.\n\n  Physical monikers are not configured in YAML. The uploader resolves the chosen\n  logical group against the current primary-moniker mapping supplied by GCS or an\n  agent-fed credential snapshot.\n\n  ## Breaking configuration change\n\n  Every Geneva exporter configuration must now provide:\n\n```yaml\n  account_routing:\n    default_group: \"<logical-account-group>\"\n```\n\n  Existing configurations must add the logical GCS account group that should\n  receive events without an explicit override.\n\n  ## Validation\n\n  - cargo xtask check\n  - cargo test -p otap-df-contrib-nodes --features geneva-exporter geneva_exporter\n      - 87 passed\n\n  - python3 tools/sanitycheck.py\n  - make chlog-validate\n  - Markdown lint\n  - git diff --check\n\n---------\n\nCo-authored-by: Utkarsh Umesan Pillai <66651184+utpilla@users.noreply.github.com>",
          "timestamp": "2026-08-19T00:30:08Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/2e9fff0f0b8808f44b663fdf27a09b2034a5096b"
        },
        "date": 1787107219694,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "log_rows_written_rate",
            "value": 99389.17247634748,
            "unit": "rows/sec",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - ClickHouse rows written"
          },
          {
            "name": "logs_produced_rate",
            "value": 100504.26738564123,
            "unit": "rows/sec",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - logs produced"
          },
          {
            "name": "logs_produced_rate",
            "value": 100367.96904812426,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - logs_produced"
          },
          {
            "name": "log_rows_written_rate",
            "value": 99929.2168047633,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - ClickHouse rows written"
          },
          {
            "name": "logs_produced_rate",
            "value": 100793.8083573052,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - logs_produced"
          },
          {
            "name": "log_rows_written_rate",
            "value": 99942.69951894248,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - ClickHouse rows written"
          }
        ]
      },
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
          "id": "2e9fff0f0b8808f44b663fdf27a09b2034a5096b",
          "message": "  feat(geneva-exporter): support account group routing (#3799)\n\n## Summary\n\n- add required `account_routing.default_group` configuration to the\nGeneva exporter\n- support optional destination event/table overrides through\n`account_routing.events`\n  - pass account routing into `geneva-uploader`\n- preserve the complete logical-group-to-primary-moniker map from\nagent-fed credentials\n- update `geneva-uploader` to the revision containing multi-moniker\nrouting\n  - update Geneva examples, documentation, tests, and changelog\n\n  This integrates the account-group routing added by\n  https://github.com/open-telemetry/opentelemetry-rust-contrib/pull/747.\n\n  ## Routing model\n\n  YAML config selects logical GCS account groups:\n\n  ```yaml\n  account_routing:\n    default_group: \"diagnostics\"\n    events:\n      AuditLogs: \"audit\"\n      SecurityEvents: \"security\"\n```\n\n  The events keys are final destination event/table names after\n  event_name_mapping has run. Events without an exact override use\n  default_group.\n\n  Physical monikers are not configured in YAML. The uploader resolves the chosen\n  logical group against the current primary-moniker mapping supplied by GCS or an\n  agent-fed credential snapshot.\n\n  ## Breaking configuration change\n\n  Every Geneva exporter configuration must now provide:\n\n```yaml\n  account_routing:\n    default_group: \"<logical-account-group>\"\n```\n\n  Existing configurations must add the logical GCS account group that should\n  receive events without an explicit override.\n\n  ## Validation\n\n  - cargo xtask check\n  - cargo test -p otap-df-contrib-nodes --features geneva-exporter geneva_exporter\n      - 87 passed\n\n  - python3 tools/sanitycheck.py\n  - make chlog-validate\n  - Markdown lint\n  - git diff --check\n\n---------\n\nCo-authored-by: Utkarsh Umesan Pillai <66651184+utpilla@users.noreply.github.com>",
          "timestamp": "2026-08-19T00:30:08Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/2e9fff0f0b8808f44b663fdf27a09b2034a5096b"
        },
        "date": 1787161996079,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "log_rows_written_rate",
            "value": 99184.78251892522,
            "unit": "rows/sec",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - ClickHouse rows written"
          },
          {
            "name": "logs_produced_rate",
            "value": 100520.83250808282,
            "unit": "rows/sec",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - logs produced"
          },
          {
            "name": "logs_produced_rate",
            "value": 99134.38709602295,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - logs_produced"
          },
          {
            "name": "log_rows_written_rate",
            "value": 99943.36542625846,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - ClickHouse rows written"
          },
          {
            "name": "logs_produced_rate",
            "value": 100255.63146927077,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - logs_produced"
          },
          {
            "name": "log_rows_written_rate",
            "value": 100087.49046300646,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - ClickHouse rows written"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Utkarsh Umesan Pillai",
            "username": "utpilla",
            "email": "66651184+utpilla@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "1f0f51b347e76a5fd58028e52db3225b2428787d",
          "message": "chore: [Geneva Exporter] Update the YAML to show events mapping example (#3824)\n\n# Change summary\n- Update the example YAML to add some comments and also show event\nmapping\n\nCo-authored-by: Utkarsh Umesan Pillai <utpilla@users.noreply.github.com>",
          "timestamp": "2026-08-19T23:12:06Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/1f0f51b347e76a5fd58028e52db3225b2428787d"
        },
        "date": 1787191102597,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "log_rows_written_rate",
            "value": 99296.64129433612,
            "unit": "rows/sec",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - ClickHouse rows written"
          },
          {
            "name": "logs_produced_rate",
            "value": 100835.6317163474,
            "unit": "rows/sec",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - logs produced"
          },
          {
            "name": "logs_produced_rate",
            "value": 98996.66377887248,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - logs_produced"
          },
          {
            "name": "log_rows_written_rate",
            "value": 99945.52968632095,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - ClickHouse rows written"
          },
          {
            "name": "logs_produced_rate",
            "value": 100458.12822635537,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - logs_produced"
          },
          {
            "name": "log_rows_written_rate",
            "value": 100079.154618019,
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
          "id": "6c815bd0bdf6c5e023daf7987e17ca5b4dda565c",
          "message": "chore(clippy): Resolve clippy 1.98 errors in main (#3835)\n\n# Chore Summary\n\nAddress clippy error on unrelated PR:\nhttps://github.com/open-telemetry/otel-arrow/actions/runs/32402148082/job/96532892549?pr=3834\n\n```text\nerror: draining all elements of a collection into a new collection of the same type\n    --> crates/core-nodes/src/processors/batch_processor/mod.rs:1485:22\n     |\n1485 |             pending: self.pending.drain(..).collect(),\n     |                      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ help: use `mem::take` to avoid creating a new allocation: `std::mem::take(&mut self.pending)`\n     |\n     = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.98.0/index.html#drain_collect\n     = note: `-D clippy::drain-collect` implied by `-D clippy::perf`\n     = help: to override `-D clippy::perf` add `#[allow(clippy::drain_collect)]`\n\nerror: draining all elements of a collection into a new collection of the same type\n    --> crates/core-nodes/src/processors/batch_processor/mod.rs:1486:22\n     |\n1486 |             context: self.context.drain(..).collect(),\n     |                      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ help: use `mem::take` to avoid creating a new allocation: `std::mem::take(&mut self.context)`\n     |\n     = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.98.0/index.html#drain_collect\n\nerror: the `Err`-variant returned from this function is very large\n   --> crates/core-nodes/src/processors/fanout_processor/mod.rs:562:10\n    |\n562 |     ) -> Result<DeadlineVec, TypedError<OtapPdata>> {\n    |          ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ the `Err`-variant is at least 152 bytes\n    |\n    = help: try reducing the size of `otap_df_engine::error::TypedError<otap_df_otap::pdata::OtapPdata>`, for example by boxing large elements or replacing it with `Box<otap_df_engine::error::TypedError<otap_df_otap::pdata::OtapPdata>>`\n    = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.98.0/index.html#result_large_err\n    = note: `-D clippy::result-large-err` implied by `-D clippy::perf`\n    = help: to override `-D clippy::perf` add `#[allow(clippy::result_large_err)]`\n```\n\n## Related issue\n\n<!-- Link the related issue if one exists. -->",
          "timestamp": "2026-08-20T19:42:52Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/6c815bd0bdf6c5e023daf7987e17ca5b4dda565c"
        },
        "date": 1787257986157,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "log_rows_written_rate",
            "value": 99128.94758563764,
            "unit": "rows/sec",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - ClickHouse rows written"
          },
          {
            "name": "logs_produced_rate",
            "value": 100792.66598395299,
            "unit": "rows/sec",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - logs produced"
          },
          {
            "name": "logs_produced_rate",
            "value": 100302.99025782134,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - logs_produced"
          },
          {
            "name": "log_rows_written_rate",
            "value": 100066.82011531966,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - ClickHouse rows written"
          },
          {
            "name": "logs_produced_rate",
            "value": 99122.60533855869,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - logs_produced"
          },
          {
            "name": "log_rows_written_rate",
            "value": 99935.70802783543,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - ClickHouse rows written"
          }
        ]
      },
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
          "id": "26b710aa0c6e5900e94523992c018104db6cfe24",
          "message": "chore(engine): add local retained-work accounting (#3756)\n\n# Change Summary\n\nAdd a runtime-local retained-work account and non-`Send` ownership\nticket.\n\nThe account tracks known retained bytes and unknown-size items. Tickets\nrefund their charge exactly once on explicit completion. Dropping an\nunresolved ticket also refunds the charge and records abandonment.\n\nChecked arithmetic reports overflow and underflow as accounting\ncorruption.\n\nThis PR does not add runtime wiring, attribution, metrics export,\nconfiguration, enforcement, escrow, or production charge sites.\n\n## Background\n\nThe retained-work pilot needs a runtime-local accounting primitive\nbefore scope wiring, metrics, or processor integration can be added.\n\n## What issue does this PR close?\n\n* Part of #3272\n\n## How are these changes tested?\n\n- `cargo check -p otap-df-engine`\n- `cargo test -p otap-df-engine retained_work::tests`\n- `cargo test -p otap-df-engine --doc`\n- `cargo clippy -p otap-df-engine --all-targets -- -D warnings`\n- `cargo xtask check`\n- `python3 tools/sanitycheck.py`\n- `git diff --check`\n\n## Are there any user-facing changes?\n\nNo. This PR adds an internal accounting primitive without changing\nruntime behavior, configuration, or exported telemetry.\n\n  ### Changelog\n\n  * [ ] Added a `.chloggen/*.yaml` entry\n  * [x] This PR is a `chore` (indicated in title)\n  * [ ] This is a documentation-only PR.",
          "timestamp": "2026-08-21T00:33:27Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/26b710aa0c6e5900e94523992c018104db6cfe24"
        },
        "date": 1787292491372,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "log_rows_written_rate",
            "value": 99123.78339883417,
            "unit": "rows/sec",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - ClickHouse rows written"
          },
          {
            "name": "logs_produced_rate",
            "value": 100767.38290300652,
            "unit": "rows/sec",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - logs produced"
          },
          {
            "name": "logs_produced_rate",
            "value": 100415.08079874377,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - logs_produced"
          },
          {
            "name": "log_rows_written_rate",
            "value": 99946.52860719516,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - ClickHouse rows written"
          },
          {
            "name": "logs_produced_rate",
            "value": 100913.75895023048,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - logs_produced"
          },
          {
            "name": "log_rows_written_rate",
            "value": 99924.55695949559,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - ClickHouse rows written"
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
          "id": "845e34cdd9da76b9ff3b1fb312fefcbe63678ddb",
          "message": "chore(repo): Update CODEOWNERS to specify required approvals for core engine crates (#3851)\n\n# Chore Summary\n\nPer offline discussion among maintainers, wanted to clarify some\nadditional CODEOWNER policies as contribution trends increase to help\nensure proper visibility to core engine crates.\n\n## Related issue\n\nN/A\n\n---------\n\nCo-authored-by: Copilot Autofix powered by AI <175728472+Copilot@users.noreply.github.com>",
          "timestamp": "2026-08-21T16:29:22Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/845e34cdd9da76b9ff3b1fb312fefcbe63678ddb"
        },
        "date": 1787334713277,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "log_rows_written_rate",
            "value": 99329.94024934377,
            "unit": "rows/sec",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - ClickHouse rows written"
          },
          {
            "name": "logs_produced_rate",
            "value": 100722.55286534152,
            "unit": "rows/sec",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - logs produced"
          },
          {
            "name": "logs_produced_rate",
            "value": 100457.63643708309,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - logs_produced"
          },
          {
            "name": "log_rows_written_rate",
            "value": 100087.65719407023,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - ClickHouse rows written"
          },
          {
            "name": "logs_produced_rate",
            "value": 99200.1041634721,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - logs_produced"
          },
          {
            "name": "log_rows_written_rate",
            "value": 99939.03718731574,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - ClickHouse rows written"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Maksat Maratov",
            "username": "maksmara",
            "email": "mmaratov@microsoft.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "b6c89a1811bef396c531145227b3130066d61c20",
          "message": "feat(console-exporter): add compact histogram percentile estimates (#3850)\n\n# Change summary\n\nAdd approximate p50, p90, and p99 values to compact pretty output for\nexplicit\nand exponential histograms.\n\nThe estimator:\n\n- uses nearest-rank bucket selection;\n- uses arithmetic midpoints for explicit buckets and geometric midpoints\nfor\n  exponential buckets;\n- handles negative, zero, and positive exponential bucket populations;\n- supports the full OTLP `sint32` scale range;\n- marks estimates with `~=` to distinguish them from exact values;\n- omits estimates when bucket data is empty, internally inconsistent, or\ncannot\n  be represented safely;\n- produces equivalent output for OTLP and OTAP views.\n\nThe PR also corrects raw OTLP ZigZag decoding so exponential histogram\n`scale`\nand bucket `offset` values can use the full `sint32` range.\n\nRaw histogram output remains unchanged.\n\n## Related issue\n\n* Closes #3840\n\n## Validation\n\n- `cargo test -p otap-df-core-nodes console_exporter`\n- `cargo test -p otap-df-pdata decodes_full_sint32_range`\n- `cargo check -p otap-df-core-nodes`\n- `cargo clippy -p otap-df-core-nodes --all-targets -- -D warnings`\n- `cargo clippy -p otap-df-pdata --all-targets -- -D warnings`\n- `npx markdownlint-cli2\nrust/otap-dataflow/crates/core-nodes/src/exporters/console_exporter/README.md`\n- `chloggen validate --config rust/otap-dataflow/.chloggen/config.yaml`\n- `cargo xtask check`\n\n## User-facing changes\n\nCompact console histogram output now includes approximate `p50~=`,\n`p90~=`,\nand `p99~=` fields when sufficient valid bucket data is available.\nIndividual\npercentiles are omitted when they cannot be estimated safely.\n\nRaw OTLP exponential histogram scale and bucket offset values are now\ndecoded\ncorrectly across the full `sint32` range.\n\nRaw histogram output is unchanged.\n\nAdded\n\n`rust/otap-dataflow/.chloggen/console-compact-histogram-percentiles.yaml`.",
          "timestamp": "2026-08-21T23:38:36Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/b6c89a1811bef396c531145227b3130066d61c20"
        },
        "date": 1787362475597,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "log_rows_written_rate",
            "value": 99153.14099765863,
            "unit": "rows/sec",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - ClickHouse rows written"
          },
          {
            "name": "logs_produced_rate",
            "value": 101040.80278534467,
            "unit": "rows/sec",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - logs produced"
          },
          {
            "name": "logs_produced_rate",
            "value": 100441.55127379461,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - logs_produced"
          },
          {
            "name": "log_rows_written_rate",
            "value": 99926.7204050363,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - ClickHouse rows written"
          },
          {
            "name": "logs_produced_rate",
            "value": 99030.58290995662,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - logs_produced"
          },
          {
            "name": "log_rows_written_rate",
            "value": 100075.82066879426,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - ClickHouse rows written"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Maksat Maratov",
            "username": "maksmara",
            "email": "mmaratov@microsoft.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "b6c89a1811bef396c531145227b3130066d61c20",
          "message": "feat(console-exporter): add compact histogram percentile estimates (#3850)\n\n# Change summary\n\nAdd approximate p50, p90, and p99 values to compact pretty output for\nexplicit\nand exponential histograms.\n\nThe estimator:\n\n- uses nearest-rank bucket selection;\n- uses arithmetic midpoints for explicit buckets and geometric midpoints\nfor\n  exponential buckets;\n- handles negative, zero, and positive exponential bucket populations;\n- supports the full OTLP `sint32` scale range;\n- marks estimates with `~=` to distinguish them from exact values;\n- omits estimates when bucket data is empty, internally inconsistent, or\ncannot\n  be represented safely;\n- produces equivalent output for OTLP and OTAP views.\n\nThe PR also corrects raw OTLP ZigZag decoding so exponential histogram\n`scale`\nand bucket `offset` values can use the full `sint32` range.\n\nRaw histogram output remains unchanged.\n\n## Related issue\n\n* Closes #3840\n\n## Validation\n\n- `cargo test -p otap-df-core-nodes console_exporter`\n- `cargo test -p otap-df-pdata decodes_full_sint32_range`\n- `cargo check -p otap-df-core-nodes`\n- `cargo clippy -p otap-df-core-nodes --all-targets -- -D warnings`\n- `cargo clippy -p otap-df-pdata --all-targets -- -D warnings`\n- `npx markdownlint-cli2\nrust/otap-dataflow/crates/core-nodes/src/exporters/console_exporter/README.md`\n- `chloggen validate --config rust/otap-dataflow/.chloggen/config.yaml`\n- `cargo xtask check`\n\n## User-facing changes\n\nCompact console histogram output now includes approximate `p50~=`,\n`p90~=`,\nand `p99~=` fields when sufficient valid bucket data is available.\nIndividual\npercentiles are omitted when they cannot be estimated safely.\n\nRaw OTLP exponential histogram scale and bucket offset values are now\ndecoded\ncorrectly across the full `sint32` range.\n\nRaw histogram output is unchanged.\n\nAdded\n\n`rust/otap-dataflow/.chloggen/console-compact-histogram-percentiles.yaml`.",
          "timestamp": "2026-08-21T23:38:36Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/b6c89a1811bef396c531145227b3130066d61c20"
        },
        "date": 1787419462049,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "log_rows_written_rate",
            "value": 99453.65169468538,
            "unit": "rows/sec",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - ClickHouse rows written"
          },
          {
            "name": "logs_produced_rate",
            "value": 100780.83453261104,
            "unit": "rows/sec",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - logs produced"
          },
          {
            "name": "logs_produced_rate",
            "value": 100972.04270935655,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - logs_produced"
          },
          {
            "name": "log_rows_written_rate",
            "value": 99937.37257984997,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - ClickHouse rows written"
          },
          {
            "name": "logs_produced_rate",
            "value": 100280.60041700753,
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
            "name": "OpenTelemetry Bot",
            "username": "opentelemetrybot",
            "email": "107717825+opentelemetrybot@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "929d7e87d6402c4a5a92b97fb0f9d4f5535b983b",
          "message": "chore: use shared OSSF Scorecard workflow (#3861)\n\nDesign discussion: open-telemetry/sig-security#309\n\n## Changes\n\nMigrate OSSF Scorecard to the shared workflow. This limits code scanning\nalerts from Scorecard to `BinaryArtifactsID`, `DangerousWorkflowID`,\n`PinnedDependenciesID`, and `TokenPermissionsID`.",
          "timestamp": "2026-08-22T18:26:47Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/929d7e87d6402c4a5a92b97fb0f9d4f5535b983b"
        },
        "date": 1787448894704,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "log_rows_written_rate",
            "value": 99363.26109732197,
            "unit": "rows/sec",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - ClickHouse rows written"
          },
          {
            "name": "logs_produced_rate",
            "value": 100901.85574152498,
            "unit": "rows/sec",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - logs produced"
          },
          {
            "name": "logs_produced_rate",
            "value": 99104.90465805818,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - logs_produced"
          },
          {
            "name": "log_rows_written_rate",
            "value": 100074.48715129953,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - ClickHouse rows written"
          },
          {
            "name": "logs_produced_rate",
            "value": 100141.7192031625,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - logs_produced"
          },
          {
            "name": "log_rows_written_rate",
            "value": 100085.15628643963,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - ClickHouse rows written"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "OpenTelemetry Bot",
            "username": "opentelemetrybot",
            "email": "107717825+opentelemetrybot@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "929d7e87d6402c4a5a92b97fb0f9d4f5535b983b",
          "message": "chore: use shared OSSF Scorecard workflow (#3861)\n\nDesign discussion: open-telemetry/sig-security#309\n\n## Changes\n\nMigrate OSSF Scorecard to the shared workflow. This limits code scanning\nalerts from Scorecard to `BinaryArtifactsID`, `DangerousWorkflowID`,\n`PinnedDependenciesID`, and `TokenPermissionsID`.",
          "timestamp": "2026-08-22T18:26:47Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/929d7e87d6402c4a5a92b97fb0f9d4f5535b983b"
        },
        "date": 1787505890324,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "log_rows_written_rate",
            "value": 99005.54818179569,
            "unit": "rows/sec",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - ClickHouse rows written"
          },
          {
            "name": "logs_produced_rate",
            "value": 100746.31676161721,
            "unit": "rows/sec",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - logs produced"
          },
          {
            "name": "logs_produced_rate",
            "value": 99125.1849608748,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - logs_produced"
          },
          {
            "name": "log_rows_written_rate",
            "value": 99944.19782288223,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - ClickHouse rows written"
          },
          {
            "name": "logs_produced_rate",
            "value": 100309.52800082666,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - logs_produced"
          },
          {
            "name": "log_rows_written_rate",
            "value": 100076.32074701722,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - ClickHouse rows written"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "OpenTelemetry Bot",
            "username": "opentelemetrybot",
            "email": "107717825+opentelemetrybot@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "929d7e87d6402c4a5a92b97fb0f9d4f5535b983b",
          "message": "chore: use shared OSSF Scorecard workflow (#3861)\n\nDesign discussion: open-telemetry/sig-security#309\n\n## Changes\n\nMigrate OSSF Scorecard to the shared workflow. This limits code scanning\nalerts from Scorecard to `BinaryArtifactsID`, `DangerousWorkflowID`,\n`PinnedDependenciesID`, and `TokenPermissionsID`.",
          "timestamp": "2026-08-22T18:26:47Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/929d7e87d6402c4a5a92b97fb0f9d4f5535b983b"
        },
        "date": 1787535293187,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "log_rows_written_rate",
            "value": 99051.92990675017,
            "unit": "rows/sec",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - ClickHouse rows written"
          },
          {
            "name": "logs_produced_rate",
            "value": 100837.16188228225,
            "unit": "rows/sec",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - logs produced"
          },
          {
            "name": "logs_produced_rate",
            "value": 100340.15647512255,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - logs_produced"
          },
          {
            "name": "log_rows_written_rate",
            "value": 99948.36001399277,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - ClickHouse rows written"
          },
          {
            "name": "logs_produced_rate",
            "value": 100833.36083843344,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - logs_produced"
          },
          {
            "name": "log_rows_written_rate",
            "value": 99939.53658036888,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - ClickHouse rows written"
          }
        ]
      }
    ]
  }
}