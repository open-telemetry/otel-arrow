window.BENCHMARK_DATA = {
  "lastUpdate": 1788400426367,
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
          "id": "3e85c3460361446ebfce99e9f35fffd2dd5ab740",
          "message": "chore(deps): update geneva-uploader digest to 43bd04b (#3867)\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n|---|---|---|---|\n| geneva-uploader | workspace.dependencies | digest | `b4cbfda` →\n`43bd04b` |\n\n---\n\n### Configuration\n\n📅 **Schedule**: (UTC)\n\n- Branch creation\n  - \"before 8am on Monday\"\n- Automerge\n  - At any time (no schedule defined)\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0NC4zOS4wIiwidXBkYXRlZEluVmVyIjoiNDQuMzkuMCIsInRhcmdldEJyYW5jaCI6Im1haW4iLCJsYWJlbHMiOlsiZGVwZW5kZW5jaWVzIl19-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2026-08-24T15:08:06Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/3e85c3460361446ebfce99e9f35fffd2dd5ab740"
        },
        "date": 1787592661899,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "log_rows_written_rate",
            "value": 99188.73095926121,
            "unit": "rows/sec",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - ClickHouse rows written"
          },
          {
            "name": "logs_produced_rate",
            "value": 99180.92692275341,
            "unit": "rows/sec",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - logs produced"
          },
          {
            "name": "logs_produced_rate",
            "value": 99256.89725664534,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - logs_produced"
          },
          {
            "name": "log_rows_written_rate",
            "value": 99944.36430387084,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - ClickHouse rows written"
          },
          {
            "name": "logs_produced_rate",
            "value": 100411.48122950447,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - logs_produced"
          },
          {
            "name": "log_rows_written_rate",
            "value": 99952.5225517879,
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
          "id": "ebc01cc6fb0c71b9830895fbdaffca89607699a5",
          "message": "feat(engine): Add opt-in `size` metric to node produced/consumed (#3842)\n\n# Change summary\n\nClosing the loop on final piece to match the [Go Collector universal\ntelemetry RFC on auto-instrumented\nmetrics](https://github.com/open-telemetry/opentelemetry-collector/blob/main/docs/rfcs/component-universal-telemetry.md#auto-instrumented-metrics)\n🥳\n\nAdds logical payload size to the engine-owned node outcome metrics:\n\n- `node.producer.produced.size{signal,outcome}`\n- `node.consumer.consumed.size{signal,outcome}`\n\nSize follows the existing item-count policy model. `runtime_metrics:\ndetailed` enables both measurements for every node. At `runtime_metrics:\nnormal`, nodes can independently opt in with:\n\n```yaml\npolicies:\n  telemetry:\n    size: true\n```\n\nThe forward path measures the payload once at each produced or consumed\nboundary and stores the value on the context frame. Ack/Nack unwinding\nrecords the stored size with the same signal and outcome as the\ncorresponding message metric.\n\nOTLP payloads report their encoded protobuf length. OTAP payloads report\nlogical Arrow bytes. Cached OTAP sizing avoids repeating the Arrow array\nand buffer walk when the payload is unchanged.\n\nThe `trafficgen-universal-produced-consumed-metrics.yaml` demo includes\nboth item and size policies and prints only produced/consumed node\nmetrics through its internal observability pipeline.\n\n### Sample config run\n\nThe `full` pipeline uses `runtime_metrics: detailed`. Its log sampler\nkeeps one third of log records while metrics and traces pass through\nunchanged:\n\n| Signal | Receiver produced | Sampler consumed | Sampler produced |\nNoop consumed |\n| --- | ---: | ---: | ---: | ---: |\n| Logs messages | 4 | 4 | 4 | 4 |\n| Logs items | 30 | 30 | 10 | 10 |\n| Logs size (By) | 7,472 | 7,472 | 6,448 | 6,448 |\n| Metrics messages | 2 | 2 | 2 | 2 |\n| Metrics items | 18 | 18 | 18 | 18 |\n| Metrics size (By) | 3,710 | 3,710 | 3,710 | 3,710 |\n| Traces messages | 2 | 2 | 2 | 2 |\n| Traces items | 12 | 12 | 12 | 12 |\n| Traces size (By) | 2,290 | 2,290 | 2,290 | 2,290 |\n\nDropping two thirds of the log records reduces logical size from `7,472\nBy` to `6,448 By` rather than by two thirds. OTAP is a columnar,\nrelational representation: resource, scope, attribute, and dictionary\nbuffers remain largely unchanged, while sampling removes primarily the\nper-record offsets and dictionary keys. The synthetic records also\nrepeat values that are stored once in dictionaries. Logical Arrow size\ntherefore describes the resulting representation, not an average record\nsize multiplied by the item count.\n\nThe `partial` pipeline uses `runtime_metrics: normal` and opts only the\nsampler into `item_counts` and `size`:\n\n| Node boundary | Messages (logs / metrics / traces) | Items (logs /\nmetrics / traces) | Size in By (logs / metrics / traces) |\n| --- | ---: | ---: | ---: |\n| Receiver produced | 4 / 2 / 2 | Not present | Not present |\n| Sampler consumed | 4 / 2 / 2 | 30 / 18 / 12 | 7,472 / 3,710 / 2,290 |\n| Sampler produced | 4 / 2 / 2 | 10 / 18 / 12 | 6,448 / 3,710 / 2,290 |\n| Noop consumed | 4 / 2 / 2 | Not present | Not present |\n\n## Related issue\n\n* Closes #2884\n\n## Validation\n\nLocal engine runs\n\n## User-facing changes\n\nYes. Users can enable node-level logical payload size metrics globally\nwith `runtime_metrics: detailed` or per node with\n`policies.telemetry.size: true`.",
          "timestamp": "2026-08-24T22:34:55Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/ebc01cc6fb0c71b9830895fbdaffca89607699a5"
        },
        "date": 1787623788076,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "log_rows_written_rate",
            "value": 98993.79633583206,
            "unit": "rows/sec",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - ClickHouse rows written"
          },
          {
            "name": "logs_produced_rate",
            "value": 100685.06459254067,
            "unit": "rows/sec",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - logs produced"
          },
          {
            "name": "logs_produced_rate",
            "value": 99045.01468333951,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - logs_produced"
          },
          {
            "name": "log_rows_written_rate",
            "value": 100077.82101167315,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - ClickHouse rows written"
          },
          {
            "name": "logs_produced_rate",
            "value": 100442.60536266083,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - logs_produced"
          },
          {
            "name": "log_rows_written_rate",
            "value": 100076.15405372092,
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
          "id": "ebc01cc6fb0c71b9830895fbdaffca89607699a5",
          "message": "feat(engine): Add opt-in `size` metric to node produced/consumed (#3842)\n\n# Change summary\n\nClosing the loop on final piece to match the [Go Collector universal\ntelemetry RFC on auto-instrumented\nmetrics](https://github.com/open-telemetry/opentelemetry-collector/blob/main/docs/rfcs/component-universal-telemetry.md#auto-instrumented-metrics)\n🥳\n\nAdds logical payload size to the engine-owned node outcome metrics:\n\n- `node.producer.produced.size{signal,outcome}`\n- `node.consumer.consumed.size{signal,outcome}`\n\nSize follows the existing item-count policy model. `runtime_metrics:\ndetailed` enables both measurements for every node. At `runtime_metrics:\nnormal`, nodes can independently opt in with:\n\n```yaml\npolicies:\n  telemetry:\n    size: true\n```\n\nThe forward path measures the payload once at each produced or consumed\nboundary and stores the value on the context frame. Ack/Nack unwinding\nrecords the stored size with the same signal and outcome as the\ncorresponding message metric.\n\nOTLP payloads report their encoded protobuf length. OTAP payloads report\nlogical Arrow bytes. Cached OTAP sizing avoids repeating the Arrow array\nand buffer walk when the payload is unchanged.\n\nThe `trafficgen-universal-produced-consumed-metrics.yaml` demo includes\nboth item and size policies and prints only produced/consumed node\nmetrics through its internal observability pipeline.\n\n### Sample config run\n\nThe `full` pipeline uses `runtime_metrics: detailed`. Its log sampler\nkeeps one third of log records while metrics and traces pass through\nunchanged:\n\n| Signal | Receiver produced | Sampler consumed | Sampler produced |\nNoop consumed |\n| --- | ---: | ---: | ---: | ---: |\n| Logs messages | 4 | 4 | 4 | 4 |\n| Logs items | 30 | 30 | 10 | 10 |\n| Logs size (By) | 7,472 | 7,472 | 6,448 | 6,448 |\n| Metrics messages | 2 | 2 | 2 | 2 |\n| Metrics items | 18 | 18 | 18 | 18 |\n| Metrics size (By) | 3,710 | 3,710 | 3,710 | 3,710 |\n| Traces messages | 2 | 2 | 2 | 2 |\n| Traces items | 12 | 12 | 12 | 12 |\n| Traces size (By) | 2,290 | 2,290 | 2,290 | 2,290 |\n\nDropping two thirds of the log records reduces logical size from `7,472\nBy` to `6,448 By` rather than by two thirds. OTAP is a columnar,\nrelational representation: resource, scope, attribute, and dictionary\nbuffers remain largely unchanged, while sampling removes primarily the\nper-record offsets and dictionary keys. The synthetic records also\nrepeat values that are stored once in dictionaries. Logical Arrow size\ntherefore describes the resulting representation, not an average record\nsize multiplied by the item count.\n\nThe `partial` pipeline uses `runtime_metrics: normal` and opts only the\nsampler into `item_counts` and `size`:\n\n| Node boundary | Messages (logs / metrics / traces) | Items (logs /\nmetrics / traces) | Size in By (logs / metrics / traces) |\n| --- | ---: | ---: | ---: |\n| Receiver produced | 4 / 2 / 2 | Not present | Not present |\n| Sampler consumed | 4 / 2 / 2 | 30 / 18 / 12 | 7,472 / 3,710 / 2,290 |\n| Sampler produced | 4 / 2 / 2 | 10 / 18 / 12 | 6,448 / 3,710 / 2,290 |\n| Noop consumed | 4 / 2 / 2 | Not present | Not present |\n\n## Related issue\n\n* Closes #2884\n\n## Validation\n\nLocal engine runs\n\n## User-facing changes\n\nYes. Users can enable node-level logical payload size metrics globally\nwith `runtime_metrics: detailed` or per node with\n`policies.telemetry.size: true`.",
          "timestamp": "2026-08-24T22:34:55Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/ebc01cc6fb0c71b9830895fbdaffca89607699a5"
        },
        "date": 1787679265892,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "log_rows_written_rate",
            "value": 99300.57734737288,
            "unit": "rows/sec",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - ClickHouse rows written"
          },
          {
            "name": "logs_produced_rate",
            "value": 100840.67678951318,
            "unit": "rows/sec",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - logs produced"
          },
          {
            "name": "logs_produced_rate",
            "value": 99117.04020659217,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - logs_produced"
          },
          {
            "name": "log_rows_written_rate",
            "value": 99939.70304582901,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - ClickHouse rows written"
          },
          {
            "name": "logs_produced_rate",
            "value": 98847.81644838389,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - logs_produced"
          },
          {
            "name": "log_rows_written_rate",
            "value": 99943.36542625846,
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
          "id": "a942658eab44e66650313ecdce8ca90a924feae7",
          "message": "fix(metrics): Simplify engine-owned node and flow metric names (#3853)\n\n# Change summary\n\nSimplify engine-owned node and flow metric names around the direction of\ndata through the pipeline:\n\n- `node.consumer.consumed.*` becomes `node.input.*`\n- `node.producer.produced.*` becomes `node.output.*`\n- `flow.consumed.*` becomes `flow.input.*`\n- `flow.produced.*` becomes `flow.output.*`\n\nAlso:\n\n- Add `messages` and logical payload `size` measurements to flow input\nand output metrics alongside `items`.\n- Report `flow.compute.duration` as a seconds-based exponential\nhistogram while retaining its processor-compute semantics.\n- Gate flow measurements with a compact interest bitmap so disabled item\nand size metrics do not inspect PData.\n- Consolidate the node-versus-flow example as\n`trafficgen-input-output-metrics.yaml`.\n- Align Rust flow metric types, fields, methods, tests, documentation,\nand configuration values with input/output terminology.\n\n## Related issue\n\n- Related to #3300\n\n### Validation\n\nThe `trafficgen-input-output-metrics.yaml` example produced the\nfollowing representative one-second delta interval. All node rows have\n`outcome=success`.\n\nThe `full` pipeline enables detailed node metrics and a one-processor\nflow around `sampler`:\n\n```text\nreceiver output\n      │\n      ├── sampler node input == flow input\n      │            30 logs in\n      │            20 logs dropped\n      │            10 logs out\n      └── sampler node output == flow output == noop node input\n```\n\nNode and flow metric sets are stacked in pipeline order. Every value\ncell is `traces / metrics / logs`.\n\n| Pipeline boundary | Metric scope | `messages` | `items` | `size` (By)\n|\n| --- | --- | ---: | ---: | ---: |\n| Receiver output | `node.output` | `1 / 2 / 4` | `6 / 18 / 30` | `1145\n/ 3710 / 7472` |\n| Sampler input | `node.input` | `1 / 2 / 4` | `6 / 18 / 30` | `1145 /\n3710 / 7472` |\n| Flow input | `flow.input` | `1 / 2 / 4` | `6 / 18 / 30` | `1145 / 3710\n/ 7472` |\n| Flow output | `flow.output` | `1 / 2 / 4` | `6 / 18 / 10` | `1145 /\n3710 / 6448` |\n| Sampler output | `node.output` | `1 / 2 / 4` | `6 / 18 / 10` | `1145 /\n3710 / 6448` |\n| Flow decision | `flow.dropped` | — | `— / — / 20` | — |\n| Noop input | `node.input` | `1 / 2 / 4` | `6 / 18 / 10` | `1145 / 3710\n/ 6448` |\n\nThe stacked rows show both boundary agreements and the sampler\ntransformation: `30 log items input - 20 dropped = 10 output`.\n\nDuration histograms use seconds. Each cell is `count / sum / min / max`.\n\n| Measurement | Traces | Metrics | Logs |\n| --- | ---: | ---: | ---: |\n| Receiver `node.output.duration` | `1 / 0.0002026 / 0.0002026 /\n0.0002026` | `2 / 0.0003946 / 0.0001809 / 0.0002137` | `4 / 0.0064401 /\n0.0009315 / 0.0022869` |\n| Sampler `node.input.duration` | `1 / 0.0001319 / 0.0001319 /\n0.0001319` | `2 / 0.0002355 / 0.0000921 / 0.0001434` | `4 / 0.005842 /\n0.0008909 / 0.0020428` |\n| Sampler `flow.compute.duration` | `1 / 0.000009 / 0.000009 / 0.000009`\n| `2 / 0.0000223 / 0.0000109 / 0.0000114` | `4 / 0.0053347 / 0.0008101 /\n0.0018651` |\n| Noop `node.input.duration` | `1 / 0.0000063 / 0.0000063 / 0.0000063` |\n`2 / 0.0000189 / 0.000009 / 0.0000099` | `4 / 0.0000316 / 0.0000062 /\n0.000009` |\n\nOver the course of implementation, I was confused about what\n`node.*.duration` actually represented, opened follow-up issue to\nimprove:\n- https://github.com/open-telemetry/otel-arrow/issues/3881\n\nThe `partial` pipeline uses normal runtime metrics and opts only the\nsampler into item and size measurements. Each signal cell remains\n`messages / items / size in bytes`; `—` means that instrument is\ndisabled.\n\n| Boundary | Scope | Traces | Metrics | Logs |\n| --- | --- | ---: | ---: | ---: |\n| Receiver output | `node.output` | `1 / — / —` | `2 / — / —` | `4 / — /\n—` |\n| Sampler input | `node.input` | `1 / 6 / 1145` | `2 / 18 / 3710` | `4 /\n30 / 7472` |\n| Sampler output | `node.output` | `1 / 6 / 1145` | `2 / 18 / 3710` | `4\n/ 10 / 6448` |\n| Noop input | `node.input` | `1 / — / —` | `2 / — / —` | `4 / — / —` |\n\nThe partial pipeline emits no flow scopes or node duration histograms.\n\nThe `no_output` pipeline uses a deterministic filter that drops every\ngenerated log and ACKs without sending. A representative snapshot\ncontained 4 input messages with 40 items and 9792 logical bytes:\n\n| Pipeline boundary | Metric scope | `messages` | `items` | `size` (By)\n|\n| --- | --- | ---: | ---: | ---: |\n| Receiver output | `node.output` | `4` | `40` | `9792` |\n| Filter input | `node.input` | `4` | `40` | `9792` |\n| Flow input | `flow.input` | `4` | `40` | `9792` |\n| Flow decision | `flow.dropped` | — | `40` | — |\n| Flow output | `flow.output` | absent | absent | absent |\n| Filter output | `node.output` | absent | absent | absent |\n| Noop input | `node.input` | absent | absent | absent |\n\n`flow.compute.duration` still records the completed processor work:\n`count=4`, `sum=0.006460706 s`, `min=0.001050001 s`, and\n`max=0.002712202 s`. This demonstrates that an ACK without a send\nfinalizes flow compute and drop accounting without inventing an output\nmessage.\n\n## User-facing changes\n\nReplace:\n\n- `node.consumer.consumed.*` with `node.input.*`\n- `node.producer.produced.*` with `node.output.*`\n- `flow.consumed.*` with `flow.input.*`\n- `flow.produced.*` with `flow.output.*`\n- Flow metric configuration values based on consumed/produced\nterminology with `input_messages`, `input_items`, `input_size`,\n`output_messages`, `output_items`, and `output_size`.\n- Scope selectors targeting the previous names with the corresponding\ninput/output scopes.\n\nTreat `flow.compute.duration` values as seconds and as histogram\nobservations.",
          "timestamp": "2026-08-25T22:23:10Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/a942658eab44e66650313ecdce8ca90a924feae7"
        },
        "date": 1787708119351,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "log_rows_written_rate",
            "value": 99314.68249642411,
            "unit": "rows/sec",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - ClickHouse rows written"
          },
          {
            "name": "logs_produced_rate",
            "value": 100700.66842372833,
            "unit": "rows/sec",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - logs produced"
          },
          {
            "name": "logs_produced_rate",
            "value": 100485.47379478687,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - logs_produced"
          },
          {
            "name": "log_rows_written_rate",
            "value": 99953.18859001034,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - ClickHouse rows written"
          },
          {
            "name": "logs_produced_rate",
            "value": 99061.01574277664,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - logs_produced"
          },
          {
            "name": "log_rows_written_rate",
            "value": 100073.82040587899,
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
          "id": "a942658eab44e66650313ecdce8ca90a924feae7",
          "message": "fix(metrics): Simplify engine-owned node and flow metric names (#3853)\n\n# Change summary\n\nSimplify engine-owned node and flow metric names around the direction of\ndata through the pipeline:\n\n- `node.consumer.consumed.*` becomes `node.input.*`\n- `node.producer.produced.*` becomes `node.output.*`\n- `flow.consumed.*` becomes `flow.input.*`\n- `flow.produced.*` becomes `flow.output.*`\n\nAlso:\n\n- Add `messages` and logical payload `size` measurements to flow input\nand output metrics alongside `items`.\n- Report `flow.compute.duration` as a seconds-based exponential\nhistogram while retaining its processor-compute semantics.\n- Gate flow measurements with a compact interest bitmap so disabled item\nand size metrics do not inspect PData.\n- Consolidate the node-versus-flow example as\n`trafficgen-input-output-metrics.yaml`.\n- Align Rust flow metric types, fields, methods, tests, documentation,\nand configuration values with input/output terminology.\n\n## Related issue\n\n- Related to #3300\n\n### Validation\n\nThe `trafficgen-input-output-metrics.yaml` example produced the\nfollowing representative one-second delta interval. All node rows have\n`outcome=success`.\n\nThe `full` pipeline enables detailed node metrics and a one-processor\nflow around `sampler`:\n\n```text\nreceiver output\n      │\n      ├── sampler node input == flow input\n      │            30 logs in\n      │            20 logs dropped\n      │            10 logs out\n      └── sampler node output == flow output == noop node input\n```\n\nNode and flow metric sets are stacked in pipeline order. Every value\ncell is `traces / metrics / logs`.\n\n| Pipeline boundary | Metric scope | `messages` | `items` | `size` (By)\n|\n| --- | --- | ---: | ---: | ---: |\n| Receiver output | `node.output` | `1 / 2 / 4` | `6 / 18 / 30` | `1145\n/ 3710 / 7472` |\n| Sampler input | `node.input` | `1 / 2 / 4` | `6 / 18 / 30` | `1145 /\n3710 / 7472` |\n| Flow input | `flow.input` | `1 / 2 / 4` | `6 / 18 / 30` | `1145 / 3710\n/ 7472` |\n| Flow output | `flow.output` | `1 / 2 / 4` | `6 / 18 / 10` | `1145 /\n3710 / 6448` |\n| Sampler output | `node.output` | `1 / 2 / 4` | `6 / 18 / 10` | `1145 /\n3710 / 6448` |\n| Flow decision | `flow.dropped` | — | `— / — / 20` | — |\n| Noop input | `node.input` | `1 / 2 / 4` | `6 / 18 / 10` | `1145 / 3710\n/ 6448` |\n\nThe stacked rows show both boundary agreements and the sampler\ntransformation: `30 log items input - 20 dropped = 10 output`.\n\nDuration histograms use seconds. Each cell is `count / sum / min / max`.\n\n| Measurement | Traces | Metrics | Logs |\n| --- | ---: | ---: | ---: |\n| Receiver `node.output.duration` | `1 / 0.0002026 / 0.0002026 /\n0.0002026` | `2 / 0.0003946 / 0.0001809 / 0.0002137` | `4 / 0.0064401 /\n0.0009315 / 0.0022869` |\n| Sampler `node.input.duration` | `1 / 0.0001319 / 0.0001319 /\n0.0001319` | `2 / 0.0002355 / 0.0000921 / 0.0001434` | `4 / 0.005842 /\n0.0008909 / 0.0020428` |\n| Sampler `flow.compute.duration` | `1 / 0.000009 / 0.000009 / 0.000009`\n| `2 / 0.0000223 / 0.0000109 / 0.0000114` | `4 / 0.0053347 / 0.0008101 /\n0.0018651` |\n| Noop `node.input.duration` | `1 / 0.0000063 / 0.0000063 / 0.0000063` |\n`2 / 0.0000189 / 0.000009 / 0.0000099` | `4 / 0.0000316 / 0.0000062 /\n0.000009` |\n\nOver the course of implementation, I was confused about what\n`node.*.duration` actually represented, opened follow-up issue to\nimprove:\n- https://github.com/open-telemetry/otel-arrow/issues/3881\n\nThe `partial` pipeline uses normal runtime metrics and opts only the\nsampler into item and size measurements. Each signal cell remains\n`messages / items / size in bytes`; `—` means that instrument is\ndisabled.\n\n| Boundary | Scope | Traces | Metrics | Logs |\n| --- | --- | ---: | ---: | ---: |\n| Receiver output | `node.output` | `1 / — / —` | `2 / — / —` | `4 / — /\n—` |\n| Sampler input | `node.input` | `1 / 6 / 1145` | `2 / 18 / 3710` | `4 /\n30 / 7472` |\n| Sampler output | `node.output` | `1 / 6 / 1145` | `2 / 18 / 3710` | `4\n/ 10 / 6448` |\n| Noop input | `node.input` | `1 / — / —` | `2 / — / —` | `4 / — / —` |\n\nThe partial pipeline emits no flow scopes or node duration histograms.\n\nThe `no_output` pipeline uses a deterministic filter that drops every\ngenerated log and ACKs without sending. A representative snapshot\ncontained 4 input messages with 40 items and 9792 logical bytes:\n\n| Pipeline boundary | Metric scope | `messages` | `items` | `size` (By)\n|\n| --- | --- | ---: | ---: | ---: |\n| Receiver output | `node.output` | `4` | `40` | `9792` |\n| Filter input | `node.input` | `4` | `40` | `9792` |\n| Flow input | `flow.input` | `4` | `40` | `9792` |\n| Flow decision | `flow.dropped` | — | `40` | — |\n| Flow output | `flow.output` | absent | absent | absent |\n| Filter output | `node.output` | absent | absent | absent |\n| Noop input | `node.input` | absent | absent | absent |\n\n`flow.compute.duration` still records the completed processor work:\n`count=4`, `sum=0.006460706 s`, `min=0.001050001 s`, and\n`max=0.002712202 s`. This demonstrates that an ACK without a send\nfinalizes flow compute and drop accounting without inventing an output\nmessage.\n\n## User-facing changes\n\nReplace:\n\n- `node.consumer.consumed.*` with `node.input.*`\n- `node.producer.produced.*` with `node.output.*`\n- `flow.consumed.*` with `flow.input.*`\n- `flow.produced.*` with `flow.output.*`\n- Flow metric configuration values based on consumed/produced\nterminology with `input_messages`, `input_items`, `input_size`,\n`output_messages`, `output_items`, and `output_size`.\n- Scope selectors targeting the previous names with the corresponding\ninput/output scopes.\n\nTreat `flow.compute.duration` values as seconds and as histogram\nobservations.",
          "timestamp": "2026-08-25T22:23:10Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/a942658eab44e66650313ecdce8ca90a924feae7"
        },
        "date": 1787766628184,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "log_rows_written_rate",
            "value": 99293.85420984232,
            "unit": "rows/sec",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - ClickHouse rows written"
          },
          {
            "name": "logs_produced_rate",
            "value": 99088.16051663957,
            "unit": "rows/sec",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - logs produced"
          },
          {
            "name": "logs_produced_rate",
            "value": 100832.55762286573,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - logs_produced"
          },
          {
            "name": "log_rows_written_rate",
            "value": 99938.70426138635,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - ClickHouse rows written"
          },
          {
            "name": "logs_produced_rate",
            "value": 100481.24726997559,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - logs_produced"
          },
          {
            "name": "log_rows_written_rate",
            "value": 99938.20487665126,
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
          "id": "a7f8706e54b01c778a8d7bbbf98f92b53aa4bc09",
          "message": "Return client-error status codes for permanent NACKs at the OTLP receivers (#3885)\n\n# Change summary\n\n### What\nWhen the pipeline permanently rejects a request because of the request\nitself (for example, telemetry that fails the Resource Validator\nProcessor with a missing or disallowed resource attribute), the OTLP\nreceivers now return a non-retryable client error (`400 Bad Request` /\ngRPC `INVALID_ARGUMENT`) instead of a retryable `503` / `UNAVAILABLE`.\n\n### Why\nA permanent NACK means \"do not retry; fix your config.\" Previously the\nOTLP/HTTP receiver returned 503 for every NACK, so clients kept\nresending data that could never succeed. gRPC returned `INTERNAL`, which\nis non-retryable but blames the server. Neither could express \"the\nclient sent bad data.\"\n\n## Related issue\n\n* Closes #3826 \n\n## Validation\n\n- Unit tests\n\n## User-facing changes\n\n| Failure | Before | After |\n| --- | --- | --- |\n| Client-caused permanent rejection | `503` / `UNAVAILABLE` (HTTP),\n`INTERNAL` (gRPC) | `400` / `INVALID_ARGUMENT` |\n| Permanent server-side failure | `503` / `INTERNAL` | `500` /\n`INTERNAL` |\n| Transient failure | `503` / `UNAVAILABLE` | `503` / `UNAVAILABLE`\n(unchanged) |\n\n---------\n\nCo-authored-by: Utkarsh Umesan Pillai <utpilla@users.noreply.github.com>",
          "timestamp": "2026-08-27T00:58:24Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/a7f8706e54b01c778a8d7bbbf98f92b53aa4bc09"
        },
        "date": 1787799411955,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "log_rows_written_rate",
            "value": 99353.41024378476,
            "unit": "rows/sec",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - ClickHouse rows written"
          },
          {
            "name": "logs_produced_rate",
            "value": 100849.72043004578,
            "unit": "rows/sec",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - logs produced"
          },
          {
            "name": "logs_produced_rate",
            "value": 100897.1420733162,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - logs_produced"
          },
          {
            "name": "log_rows_written_rate",
            "value": 100076.4874408688,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - ClickHouse rows written"
          },
          {
            "name": "logs_produced_rate",
            "value": 100562.98005519334,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - logs_produced"
          },
          {
            "name": "log_rows_written_rate",
            "value": 100084.48939884594,
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
          "id": "51d89ded5ad4a169eaa4ba422368e0913f74a738",
          "message": "fix(azure_monitor_exporter): Return permanant NACK for an empty batch (#3903)\n\n# Change summary\n\nMinor follow-up from #3891\n\nTechnically there is no reason to return a retryable NACK from the\nexporter if there is an empty payload detected. While we are fixing this\nspecific issue upstream, it is best for this component to also handle it\ncorrectly.\n\n## Related issue\n\n* Related to #3891\n\n## Validation\n\nUnit test\n\n## User-facing changes\n\nAzure Monitor exporter now permanently rejects empty batches instead of\nallowing them to be retried.",
          "timestamp": "2026-08-27T17:31:15Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/51d89ded5ad4a169eaa4ba422368e0913f74a738"
        },
        "date": 1787863981592,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "log_rows_written_rate",
            "value": 99132.71160110083,
            "unit": "rows/sec",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - ClickHouse rows written"
          },
          {
            "name": "logs_produced_rate",
            "value": 99132.89821283556,
            "unit": "rows/sec",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - logs produced"
          },
          {
            "name": "logs_produced_rate",
            "value": 100831.91200950253,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - logs_produced"
          },
          {
            "name": "log_rows_written_rate",
            "value": 100065.32018047156,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - ClickHouse rows written"
          },
          {
            "name": "logs_produced_rate",
            "value": 100227.50284127984,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - logs_produced"
          },
          {
            "name": "log_rows_written_rate",
            "value": 100084.82284153187,
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
          "id": "80d38834e747127ee02559e8c6dafd792193bdb2",
          "message": "chore(repo): Isolate test-only nodes in `dev-nodes` crate to minimize dependency chain (#3912)\n\n# Chore summary\n\nMoves the following nodes into a new `dev-nodes` crate:\n- `traffic_generator_receiver`\n- `delay_processor`\n- `error_exporter`\n- `perf_exporter`\n\nAs #3909 mentions, we do this to unblock publishing of `core-nodes`\ncrate on `crates.io` because there is currently a git Weaver dependency\nwith no published crate. We do not plan to publish `dev-nodes`\nexternally.\n\n## Related issue\n\n- Closes #3909",
          "timestamp": "2026-08-28T01:54:22Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/80d38834e747127ee02559e8c6dafd792193bdb2"
        },
        "date": 1787890706304,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "log_rows_written_rate",
            "value": 99199.5295042779,
            "unit": "rows/sec",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - ClickHouse rows written"
          },
          {
            "name": "logs_produced_rate",
            "value": 100704.94468327741,
            "unit": "rows/sec",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - logs produced"
          },
          {
            "name": "logs_produced_rate",
            "value": 100425.51653055917,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - logs_produced"
          },
          {
            "name": "log_rows_written_rate",
            "value": 100075.65397716389,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - ClickHouse rows written"
          },
          {
            "name": "logs_produced_rate",
            "value": 100819.66556134146,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - logs_produced"
          },
          {
            "name": "log_rows_written_rate",
            "value": 100073.48703650034,
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
          "id": "2edcaaf8e6a6e3c85f52abdbace9a7f39f774be5",
          "message": "feat(kafka): replay transiently nacked records (#3928)\n\n# Change summary\n\nAdds reliable transient-NACK replay to the Kafka receiver:\n\n- Manual-commit mode now replays transiently NACKed records by default.\n- Replay pauses only the affected partition and uses capped backoff.\n- Rebalance handling prevents partitions from remaining paused after\nreassignment, including retrying failed resume operations.\n- Replay logic and integration tests are isolated in dedicated\nsubmodules.\n- Receiver instrumentation and documentation are updated, including\nplanned future DLQ support.\n- End-to-end coverage verifies retry-processor exhaustion falls back to\nKafka replay.\n\n## Related issue\n\n- Related to #3505\n\n## Validation\n\n- End-to-end Kafka receiver → retry processor → exporter replay test\npassed.\n- Rebalance, reassignment, ingress-drain, and shutdown tests passed.\n\n## User-facing changes\n\nThis is a breaking behavioral change for manual-commit configurations:\ntransient NACKs now preserve the offset and replay the record by\ndefault.\n\nTo retain the previous offset-advancing behavior, configure:\n\n```yaml\ntransient_nack:\n  mode: commit_and_skip\n```\n\nA changelog entry is included.",
          "timestamp": "2026-08-28T20:40:04Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/2edcaaf8e6a6e3c85f52abdbace9a7f39f774be5"
        },
        "date": 1787951996680,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "log_rows_written_rate",
            "value": 99361.4549612745,
            "unit": "rows/sec",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - ClickHouse rows written"
          },
          {
            "name": "logs_produced_rate",
            "value": 100893.54688237257,
            "unit": "rows/sec",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - logs produced"
          },
          {
            "name": "logs_produced_rate",
            "value": 100767.34839640606,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - logs_produced"
          },
          {
            "name": "log_rows_written_rate",
            "value": 100074.65383904286,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - ClickHouse rows written"
          },
          {
            "name": "logs_produced_rate",
            "value": 100476.17164886974,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - logs_produced"
          },
          {
            "name": "log_rows_written_rate",
            "value": 100073.32035264392,
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
          "id": "a3d3101072d2b7e0e0bdbca175ec981854aff015",
          "message": "fix(quiver): Fix semantics of loss metrics and introduce contrasting reclaimed metrics (#3898)\n\n# Change summary\n\n- report durable-buffer retention loss from only the bundles that remain\nunresolved when a segment is removed\n- persist optional logical byte counts per bundle so loss bytes reflect\nthe original OTAP or OTLP payload rather than the containing segment\nfile\n- separate physical storage reclamation into\n`processor.durable_buffer.reclaimed`\n- restore subscriber progress before startup expiry accounting and\nreport deferred file deletion only after removal is confirmed\n\n### Details\n\nQuiver stores multiple bundles in each physical segment. Previously, if\none unresolved bundle kept a segment until retention expiry, the entire\nsegment was reported as lost—including bundles that downstream exporters\nhad already ACKed.\n\n```text\nsegment selected by retention\n├── bundle 0: unresolved  -> logical loss\n└── bundle 1: ACKed       -> excluded from logical loss\n\nphysical segment removal  -> reclaimed storage\n```\n\nRetention now snapshots the union of unresolved bundle indices across\nsubscribers while force-completing the segment under the same\nsubscriber-state lock. Each stored bundle is counted once when any\nsubscriber still needs it, avoiding both ACK races and per-subscriber\ndouble counting.\n\nThe segment manifest now includes an optional nullable `byte_count` for\neach bundle:\n\n- OTAP records use their active-range Arrow logical size\n- OTLP pass-through records use encoded protobuf wire length\n- known zero remains distinct from an unavailable count\n- loss bytes are emitted only when the selected unresolved bundles all\nhave exact counts\n\nThe manifest change is additive. New readers treat a missing\n`byte_count` column as unavailable, while existing readers continue\nlocating known fields by name and ignore the additional column.\n\n## Related issue\n\n* Closes #3892 \n\n## Validation\n\nRegression coverage exercises:\n\n- a partially ACKed segment, verifying only its unresolved bundle\ncontributes bundles, items, and logical bytes to expiry loss\n- a fully ACKed later segment retained behind an incomplete predecessor,\nverifying resolved data is excluded from expiry loss\n- multiple subscribers with overlapping unresolved bundles, verifying\nunion semantics count each stored bundle once\n- startup expiry after subscriber progress restoration, verifying\npersisted ACKs remain excluded\n- immediate, deferred, retried, and abandoned physical deletion,\nverifying reclaimed counters advance only after confirmed removal\n- nullable manifest byte counts, including known zero, known nonzero,\nunavailable counts, and legacy manifests without the new column\n- canonical OTAP active-range and OTLP protobuf-wire byte measurements\n\nA standalone mixed-segment repro sends one log bundle and one metric\nbundle through the same durable buffer, ACKs one downstream, transiently\nNACKs the other, and waits for the segment to expire.\n\n### Single-item batches\n\n| Unresolved bundle | `reclaimed.segments` | `reclaimed.bytes` |\n`loss.bundles` | `loss.bytes` | `loss.items` |\n| --- | ---: | ---: | ---: | ---: | ---: |\n| 1 log | 1 `reason=expired` | 7,344 `reason=expired` | 1\n`reason=expired` | 359 `reason=expired` | 1 `signal=logs,\nreason=expired` |\n| 1 metric datapoint | 1 `reason=expired` | 7,344 `reason=expired` | 1\n`reason=expired` | 329 `reason=expired` | 1 `signal=metrics,\nreason=expired` |\n\n### 100-item batches\n\n| Unresolved bundle | `reclaimed.segments` | `reclaimed.bytes` |\n`loss.bundles` | `loss.bytes` | `loss.items` |\n| --- | ---: | ---: | ---: | ---: | ---: |\n| 100 logs | 1 `reason=expired` | 57,840 `reason=expired` | 1\n`reason=expired` | 24,248 `reason=expired` | 100 `signal=logs,\nreason=expired` |\n| 100 metric datapoints | 1 `reason=expired` | 57,840 `reason=expired` |\n1 `reason=expired` | 26,952 `reason=expired` | 100 `signal=metrics,\nreason=expired` |\n\nFor the 100-item run, the segment contained 51,200 logical payload bytes\nand 6,640 bytes of Arrow IPC and Quiver segment overhead:\n\n```text\n57,840 physical bytes reclaimed\n├── 24,248 logical log bytes\n├── 26,952 logical metric bytes\n└──  6,640 segment encoding overhead\n```\n\nIn every permutation, the physical segment size remained constant while\n`loss.bytes` changed to match only the unresolved bundle. The ACKed\nneighboring bundle was excluded from logical loss.\n\n## User-facing changes\n\n| Previous metric | Replacement | Meaning |\n| --- | --- | --- |\n| `processor.durable_buffer.loss.segments` |\n`processor.durable_buffer.reclaimed.segments` | Physical segment files\nremoved |\n| `processor.durable_buffer.loss.bytes` |\n`processor.durable_buffer.reclaimed.bytes` | Physical persisted bytes\nremoved |\n| N/A | `processor.durable_buffer.loss.bytes` | Logical bytes in\nunresolved bundles |\n| `processor.durable_buffer.loss.bundles` | unchanged | Unresolved\nbundles removed by retention |\n| `processor.durable_buffer.loss.items` | unchanged | Items in\nunresolved bundles, partitioned by signal |\n\nPhysical reclamation advances only after a segment file is confirmed\nremoved. Deferred or abandoned deletion attempts are not reported as\nreclaimed.",
          "timestamp": "2026-08-28T23:29:54Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/a3d3101072d2b7e0e0bdbca175ec981854aff015"
        },
        "date": 1787968188022,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "log_rows_written_rate",
            "value": 99264.46837035457,
            "unit": "rows/sec",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - ClickHouse rows written"
          },
          {
            "name": "logs_produced_rate",
            "value": 100726.95497020258,
            "unit": "rows/sec",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - logs produced"
          },
          {
            "name": "logs_produced_rate",
            "value": 99081.63250054885,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - logs_produced"
          },
          {
            "name": "log_rows_written_rate",
            "value": 99935.20867304364,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - ClickHouse rows written"
          },
          {
            "name": "logs_produced_rate",
            "value": 100472.64032937042,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - logs_produced"
          },
          {
            "name": "log_rows_written_rate",
            "value": 99931.5468903801,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - ClickHouse rows written"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Swapnil Ashtekar",
            "username": "swashtek",
            "email": "46826200+swashtek@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "3065676427079b08049c6167d0d7fd410ff25743",
          "message": "fix(otlp_http_exporter): enhance error handling for HTTP status responses (#3902)\n\n# Change summary\nThe OTLP/HTTP exporter `usesreqwest::Response::error_for_status()` to\ndetect non-2xx responses. This call discards the response body, so any\nexplanation the backend sent back (missing/invalid field, rejection\nreason, rate-limit detail, etc.) was silently lost before it ever\nreached logs or NACK reasons.\n\nI would like to change it to include an explicit status check and add\n`RpcStatus` or falls back to raw `UTF-8` text.\n\n> Chore PR? Open **Preview**, then [use the chore\ntemplate](?template=chore.md).\n\n<!--Replace with a brief summary of the change in this PR-->\n\n## Related issue\nNone\n\n## Validation\nValidated locally while debugging issues like `400 Bad Request`, `503\nServer Unavailable` etc.\n\n## User-facing changes\n\n<!--\nDescribe the impact, or write `None`.\nUser-facing changes require a `.chloggen/*.yaml` entry. If no entry is\nneeded,\ninclude `chore` in the PR title. Documentation-only changes are exempt.\n-->",
          "timestamp": "2026-08-29T06:01:56Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/3065676427079b08049c6167d0d7fd410ff25743"
        },
        "date": 1788024544445,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "log_rows_written_rate",
            "value": 99033.63333349826,
            "unit": "rows/sec",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - ClickHouse rows written"
          },
          {
            "name": "logs_produced_rate",
            "value": 100890.8594488753,
            "unit": "rows/sec",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - logs produced"
          },
          {
            "name": "logs_produced_rate",
            "value": 100357.03119475844,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - logs_produced"
          },
          {
            "name": "log_rows_written_rate",
            "value": 99934.70932324215,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - ClickHouse rows written"
          },
          {
            "name": "logs_produced_rate",
            "value": 100575.63290108553,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - logs_produced"
          },
          {
            "name": "log_rows_written_rate",
            "value": 99937.70549690693,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - ClickHouse rows written"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Swapnil Ashtekar",
            "username": "swashtek",
            "email": "46826200+swashtek@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "3065676427079b08049c6167d0d7fd410ff25743",
          "message": "fix(otlp_http_exporter): enhance error handling for HTTP status responses (#3902)\n\n# Change summary\nThe OTLP/HTTP exporter `usesreqwest::Response::error_for_status()` to\ndetect non-2xx responses. This call discards the response body, so any\nexplanation the backend sent back (missing/invalid field, rejection\nreason, rate-limit detail, etc.) was silently lost before it ever\nreached logs or NACK reasons.\n\nI would like to change it to include an explicit status check and add\n`RpcStatus` or falls back to raw `UTF-8` text.\n\n> Chore PR? Open **Preview**, then [use the chore\ntemplate](?template=chore.md).\n\n<!--Replace with a brief summary of the change in this PR-->\n\n## Related issue\nNone\n\n## Validation\nValidated locally while debugging issues like `400 Bad Request`, `503\nServer Unavailable` etc.\n\n## User-facing changes\n\n<!--\nDescribe the impact, or write `None`.\nUser-facing changes require a `.chloggen/*.yaml` entry. If no entry is\nneeded,\ninclude `chore` in the PR title. Documentation-only changes are exempt.\n-->",
          "timestamp": "2026-08-29T06:01:56Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/3065676427079b08049c6167d0d7fd410ff25743"
        },
        "date": 1788054949442,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "log_rows_written_rate",
            "value": 99369.50098164294,
            "unit": "rows/sec",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - ClickHouse rows written"
          },
          {
            "name": "logs_produced_rate",
            "value": 100908.94234360717,
            "unit": "rows/sec",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - logs produced"
          },
          {
            "name": "logs_produced_rate",
            "value": 100808.1131175902,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - logs_produced"
          },
          {
            "name": "log_rows_written_rate",
            "value": 100078.32110988775,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - ClickHouse rows written"
          },
          {
            "name": "logs_produced_rate",
            "value": 100312.37943705895,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - logs_produced"
          },
          {
            "name": "log_rows_written_rate",
            "value": 99939.70304582901,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - ClickHouse rows written"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Swapnil Ashtekar",
            "username": "swashtek",
            "email": "46826200+swashtek@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "3065676427079b08049c6167d0d7fd410ff25743",
          "message": "fix(otlp_http_exporter): enhance error handling for HTTP status responses (#3902)\n\n# Change summary\nThe OTLP/HTTP exporter `usesreqwest::Response::error_for_status()` to\ndetect non-2xx responses. This call discards the response body, so any\nexplanation the backend sent back (missing/invalid field, rejection\nreason, rate-limit detail, etc.) was silently lost before it ever\nreached logs or NACK reasons.\n\nI would like to change it to include an explicit status check and add\n`RpcStatus` or falls back to raw `UTF-8` text.\n\n> Chore PR? Open **Preview**, then [use the chore\ntemplate](?template=chore.md).\n\n<!--Replace with a brief summary of the change in this PR-->\n\n## Related issue\nNone\n\n## Validation\nValidated locally while debugging issues like `400 Bad Request`, `503\nServer Unavailable` etc.\n\n## User-facing changes\n\n<!--\nDescribe the impact, or write `None`.\nUser-facing changes require a `.chloggen/*.yaml` entry. If no entry is\nneeded,\ninclude `chore` in the PR title. Documentation-only changes are exempt.\n-->",
          "timestamp": "2026-08-29T06:01:56Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/3065676427079b08049c6167d0d7fd410ff25743"
        },
        "date": 1788110979462,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "log_rows_written_rate",
            "value": 99298.28153951337,
            "unit": "rows/sec",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - ClickHouse rows written"
          },
          {
            "name": "logs_produced_rate",
            "value": 100884.73728765294,
            "unit": "rows/sec",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - logs produced"
          },
          {
            "name": "logs_produced_rate",
            "value": 100774.69200818737,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - logs_produced"
          },
          {
            "name": "log_rows_written_rate",
            "value": 100085.6564579672,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - ClickHouse rows written"
          },
          {
            "name": "logs_produced_rate",
            "value": 100487.49207490066,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - logs_produced"
          },
          {
            "name": "log_rows_written_rate",
            "value": 99939.37011546329,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - ClickHouse rows written"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Swapnil Ashtekar",
            "username": "swashtek",
            "email": "46826200+swashtek@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "3065676427079b08049c6167d0d7fd410ff25743",
          "message": "fix(otlp_http_exporter): enhance error handling for HTTP status responses (#3902)\n\n# Change summary\nThe OTLP/HTTP exporter `usesreqwest::Response::error_for_status()` to\ndetect non-2xx responses. This call discards the response body, so any\nexplanation the backend sent back (missing/invalid field, rejection\nreason, rate-limit detail, etc.) was silently lost before it ever\nreached logs or NACK reasons.\n\nI would like to change it to include an explicit status check and add\n`RpcStatus` or falls back to raw `UTF-8` text.\n\n> Chore PR? Open **Preview**, then [use the chore\ntemplate](?template=chore.md).\n\n<!--Replace with a brief summary of the change in this PR-->\n\n## Related issue\nNone\n\n## Validation\nValidated locally while debugging issues like `400 Bad Request`, `503\nServer Unavailable` etc.\n\n## User-facing changes\n\n<!--\nDescribe the impact, or write `None`.\nUser-facing changes require a `.chloggen/*.yaml` entry. If no entry is\nneeded,\ninclude `chore` in the PR title. Documentation-only changes are exempt.\n-->",
          "timestamp": "2026-08-29T06:01:56Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/3065676427079b08049c6167d0d7fd410ff25743"
        },
        "date": 1788141368274,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "log_rows_written_rate",
            "value": 99309.4336007822,
            "unit": "rows/sec",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - ClickHouse rows written"
          },
          {
            "name": "logs_produced_rate",
            "value": 100879.97603091769,
            "unit": "rows/sec",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - logs produced"
          },
          {
            "name": "logs_produced_rate",
            "value": 100259.72707127164,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - logs_produced"
          },
          {
            "name": "log_rows_written_rate",
            "value": 99943.36542625846,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - ClickHouse rows written"
          },
          {
            "name": "logs_produced_rate",
            "value": 100920.23612711327,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - logs_produced"
          },
          {
            "name": "log_rows_written_rate",
            "value": 100080.65496764307,
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
          "id": "e8a40bb244fe579134cbe0ed091c8890bb5a6097",
          "message": "chore(deps): update geneva-uploader digest to 91057e4 (#3948)\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n|---|---|---|---|\n| geneva-uploader | workspace.dependencies | digest | `43bd04b` →\n`91057e4` |\n\n---\n\n> [!WARNING]\n> Some dependencies could not be looked up. Check the [Dependency\nDashboard](../issues/417) for more information.\n\n---\n\n### Configuration\n\n📅 **Schedule**: (UTC)\n\n- Branch creation\n  - \"before 8am on Monday\"\n- Automerge\n  - At any time (no schedule defined)\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0NC40OS4wIiwidXBkYXRlZEluVmVyIjoiNDQuNDkuMCIsInRhcmdldEJyYW5jaCI6Im1haW4iLCJsYWJlbHMiOlsiZGVwZW5kZW5jaWVzIl19-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2026-08-31T14:29:38Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/e8a40bb244fe579134cbe0ed091c8890bb5a6097"
        },
        "date": 1788197448576,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "log_rows_written_rate",
            "value": 99256.42839668613,
            "unit": "rows/sec",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - ClickHouse rows written"
          },
          {
            "name": "logs_produced_rate",
            "value": 100894.55636032975,
            "unit": "rows/sec",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - logs produced"
          },
          {
            "name": "logs_produced_rate",
            "value": 100346.34037649244,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - logs_produced"
          },
          {
            "name": "log_rows_written_rate",
            "value": 99925.55546118142,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - ClickHouse rows written"
          },
          {
            "name": "logs_produced_rate",
            "value": 99165.95215357101,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - logs_produced"
          },
          {
            "name": "log_rows_written_rate",
            "value": 99936.04093380236,
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
          "id": "801f2bbdff14bf0ceea000d8314a434d371cbe7c",
          "message": "chore(repo): Rename rust/experimental to rust/contrib (#3953)\n\n# Chore Summary\n\nUse proper terminology now that we released\n`otel-arrow-contrib-data-engine` crates.\n\n## Related issue\n\nN/A",
          "timestamp": "2026-08-31T19:34:47Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/801f2bbdff14bf0ceea000d8314a434d371cbe7c"
        },
        "date": 1788227942867,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "log_rows_written_rate",
            "value": 99356.52946940367,
            "unit": "rows/sec",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - ClickHouse rows written"
          },
          {
            "name": "logs_produced_rate",
            "value": 100862.39533362146,
            "unit": "rows/sec",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - logs produced"
          },
          {
            "name": "logs_produced_rate",
            "value": 100684.60666106868,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - logs_produced"
          },
          {
            "name": "log_rows_written_rate",
            "value": 100075.32059556904,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - ClickHouse rows written"
          },
          {
            "name": "logs_produced_rate",
            "value": 98747.7513128597,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - logs_produced"
          },
          {
            "name": "log_rows_written_rate",
            "value": 99948.36001399277,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - ClickHouse rows written"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "uros-stefanovic-db",
            "username": "uros-stefanovic-db",
            "email": "uros.stefanovic@databricks.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "408dd1982aa0e936e98ec135a173e71564721ed7",
          "message": "Avoid quadratic resource/scope lookups in OTAP trace view (#3936)\n\n`OtapTracesView` looks up the first spans-batch row for each resource\nand scope (to read the resource and scope name, version, and\ndropped-attribute counts) by scanning `resource_groups` and\n`scope_groups_map` on every call. Those accessors run once per resource\nand once per scope, so on a batch with G distinct resources and S\ndistinct scopes the repeated scans add up to O(G^2) and O(S^2).\n\nEach `OtapResourceSpansView` / `OtapScopeSpansView` is already handed\nthe `RowGroup` for its id when the view is built. The fix carries that\ngroup's first row onto the resource/scope view and reads it directly in\nthe accessors, so each lookup is O(1) with no extra maps and no derived\nstate to keep in sync — the grouping the view already does stays the\nonly pass over the rows.\n\nThis should not change any behavior. Resource and scope ids are assigned\nmonotonically per batch, so each id maps to exactly one group, and that\ngroup's first row is the same row the previous scan resolved. The\nexisting `views::otap::traces` unit tests already exercise\nmulti-resource and multi-scope batches.\n\nI only touched the traces view here. `metrics.rs` has the same pattern,\nso I can follow up with the same change there if that's useful.\n\n---------\n\nSigned-off-by: Uros Stefanovic <uros.stefanovic@databricks.com>\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-09-01T15:52:57Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/408dd1982aa0e936e98ec135a173e71564721ed7"
        },
        "date": 1788285493672,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "log_rows_written_rate",
            "value": 99531.84875215395,
            "unit": "rows/sec",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - ClickHouse rows written"
          },
          {
            "name": "logs_produced_rate",
            "value": 99033.2916460091,
            "unit": "rows/sec",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - logs produced"
          },
          {
            "name": "logs_produced_rate",
            "value": 99304.34946823111,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - logs_produced"
          },
          {
            "name": "log_rows_written_rate",
            "value": 100081.48851465028,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - ClickHouse rows written"
          },
          {
            "name": "logs_produced_rate",
            "value": 100296.83272631934,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - logs_produced"
          },
          {
            "name": "log_rows_written_rate",
            "value": 100079.82143452045,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - ClickHouse rows written"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "otelbot-arrow[bot]",
            "username": "otelbot-arrow[bot]",
            "email": "289780372+otelbot-arrow[bot]@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "c5bda0a3cb6700ee1e455dea29ef229eb16c9fcf",
          "message": "chore(release) Prepare Release v0.54.0 (#3970)\n\n## Release v0.54.0\n\nThis PR prepares the repository for release v0.54.0.\n\n### Changes included:\n- Rendered pending chloggen entries into `go/CHANGELOG.md` and\n`rust/otap-dataflow/CHANGELOG.md`\n- Bumped `rust/otap-dataflow/Cargo.toml` (workspace + root package)\n\n## What's Changed (Go :hamster:)\n\nNo changes. This release maintains version parity across the repository.\n\n## What's Changed (Rust :crab:)\n\n### :stop_sign: Breaking changes :stop_sign:\n\n- `pipeline`: Migrated temporal reaggregation processor telemetry from\nthree flat counters to three dimensioned metric populations:\n`operations`, `failures{error.type}`, and `flushes{outcome,reason}`.\n([#3530](https://github.com/open-telemetry/otel-arrow/issues/3530))\n\n### :bulb: Enhancements :bulb:\n\n- `dependencies`: Upgrade various Rust dependencies.\n([#3947](https://github.com/open-telemetry/otel-arrow/issues/3947))\n- `engine`: Publish the telemetry, state, engine, admin, and controller\ncrates as versioned crates.io packages.\n([#1340](https://github.com/open-telemetry/otel-arrow/issues/1340))\n- `otap`: Avoid quadratic resource and scope scans in the OTAP trace\nview by resolving each resource's and scope's representative row from\nthe pre-computed row group\n([#3936](https://github.com/open-telemetry/otel-arrow/issues/3936))\n- `otap`: Avoid quadratic resource and scope scans in the OTAP metrics\nview by resolving each resource's and scope's representative row from\nthe pre-computed row group\n([#3964](https://github.com/open-telemetry/otel-arrow/issues/3964))\n\n### Checklist:\n- [ ] Verify both CHANGELOG.md files render the expected entries\n- [ ] Verify Rust crate versions updated\n- [ ] Confirm all tests pass\n- [ ] Ready to merge and tag release\n\nAfter merging this PR, run the **Push Release** workflow to create git\ntags and publish the GitHub release.\n\nCo-authored-by: otelbot-arrow[bot] <289780372+otelbot-arrow[bot]@users.noreply.github.com>",
          "timestamp": "2026-09-01T22:41:09Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/c5bda0a3cb6700ee1e455dea29ef229eb16c9fcf"
        },
        "date": 1788313994736,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "log_rows_written_rate",
            "value": 99262.54622292657,
            "unit": "rows/sec",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - ClickHouse rows written"
          },
          {
            "name": "logs_produced_rate",
            "value": 100645.03573990511,
            "unit": "rows/sec",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - logs produced"
          },
          {
            "name": "logs_produced_rate",
            "value": 98995.95284410004,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - logs_produced"
          },
          {
            "name": "log_rows_written_rate",
            "value": 100082.32207554241,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - ClickHouse rows written"
          },
          {
            "name": "logs_produced_rate",
            "value": 100464.94666477745,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - logs_produced"
          },
          {
            "name": "log_rows_written_rate",
            "value": 100072.98698659685,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - ClickHouse rows written"
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
          "id": "11b92ef5b8a51aea55be14a84c9232760a6f5095",
          "message": "fix(deps): update module google.golang.org/grpc to v1.83.1 [security] (#3974)\n\nThis PR contains the following updates:\n\n| Package | Change |\n[Age](https://docs.renovatebot.com/merge-confidence/) |\n[Confidence](https://docs.renovatebot.com/merge-confidence/) |\n|---|---|---|---|\n| [google.golang.org/grpc](https://redirect.github.com/grpc/grpc-go) |\n`v1.82.1` → `v1.83.1` |\n![age](https://developer.mend.io/api/mc/badges/age/go/google.golang.org%2fgrpc/v1.83.1?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/go/google.golang.org%2fgrpc/v1.82.1/v1.83.1?slim=true)\n|\n\n---\n\n> [!WARNING]\n> Some dependencies could not be looked up. Check the [Dependency\nDashboard](../issues/417) for more information.\n\n---\n\n### gRPC-Go: Heap Memory Exhaustion (OOM) via HTTP/2 DATA Frame\nFragmentation\n[CVE-2026-84304](https://nvd.nist.gov/vuln/detail/CVE-2026-84304) /\n[GHSA-vp52-pcj8-j9qc](https://redirect.github.com/advisories/GHSA-vp52-pcj8-j9qc)\n\n<details>\n<summary>More information</summary>\n\n#### Details\n##### Impact\nAn unauthenticated remote attacker can initiate a gRPC stream and\npurposefully fragment their payload into millions of tiny (e.g., 1-byte)\nHTTP/2 DATA frames. Even if the total payload volume falls within the\nconfigured connection and stream flow-control windows, each independent\nfragment incurs memory overhead due to internal tracking structures and\nqueue allocation.\n\nRepeated fragmentation massively inflates the heap space consumed by the\nstream. An attacker multiplexing multiple concurrent streams can exhaust\nthe memory bounds of the runtime, forcing a runtime panic or OutOfMemory\ncondition and leading to a remote Denial of Service (DoS).\n\n##### Patches\nThe change to fix this issue is merged in `master` and a patch release,\n1.83.1, has been published that contains this fix.\n\n##### Workarounds\nThis vulnerability is mitigated by implementing receive buffer\ncompaction. Consecutive small data buffers are automatically coalesced\ninto larger buffers from a shared pool once the overhead is perceived to\nbe excessive relative to actual payload data, drastically minimizing\nper-frame memory overheads.\n\nThis behavior is enabled by default. A temporary escape hatch is\nprovided via the environment variable\n`GRPC_GO_EXPERIMENTAL_ENABLE_RECEIVE_BUFFER_COMPACTION=false` to disable\nthe feature if unforeseen issues arise, but it will be removed in a\nfuture release.\n\n#### Severity\n- CVSS Score: 8.7 / 10 (High)\n- Vector String:\n`CVSS:4.0/AV:N/AC:L/AT:N/PR:N/UI:N/VC:N/VI:N/VA:H/SC:N/SI:N/SA:N`\n\n#### References\n-\n[https://github.com/grpc/grpc-go/security/advisories/GHSA-vp52-pcj8-j9qc](https://redirect.github.com/grpc/grpc-go/security/advisories/GHSA-vp52-pcj8-j9qc)\n-\n[https://nvd.nist.gov/vuln/detail/CVE-2026-84304](https://nvd.nist.gov/vuln/detail/CVE-2026-84304)\n-\n[https://github.com/grpc/grpc-go/pull/9331](https://redirect.github.com/grpc/grpc-go/pull/9331)\n-\n[https://github.com/grpc/grpc-go/pull/9333](https://redirect.github.com/grpc/grpc-go/pull/9333)\n-\n[https://github.com/grpc/grpc-go/commit/7354d9c8debb4bcf2225bf429857078de310c176](https://redirect.github.com/grpc/grpc-go/commit/7354d9c8debb4bcf2225bf429857078de310c176)\n-\n[https://github.com/grpc/grpc-go/commit/8cfeca0e1ee5ea0980dcc320e20240fa1079ec77](https://redirect.github.com/grpc/grpc-go/commit/8cfeca0e1ee5ea0980dcc320e20240fa1079ec77)\n-\n[https://github.com/grpc/grpc-go/releases/tag/v1.83.1](https://redirect.github.com/grpc/grpc-go/releases/tag/v1.83.1)\n-\n[https://github.com/advisories/GHSA-vp52-pcj8-j9qc](https://redirect.github.com/advisories/GHSA-vp52-pcj8-j9qc)\n\nThis data is provided by the [GitHub Advisory\nDatabase](https://redirect.github.com/advisories/GHSA-vp52-pcj8-j9qc)\n([CC-BY\n4.0](https://redirect.github.com/github/advisory-database/blob/main/LICENSE.md)).\n</details>\n\n---\n\n### gRPC-Go: Heap Memory Exhaustion (OOM) via HTTP/2 DATA Frame\nFragmentation\n[CVE-2026-84304](https://nvd.nist.gov/vuln/detail/CVE-2026-84304) /\n[GHSA-vp52-pcj8-j9qc](https://redirect.github.com/advisories/GHSA-vp52-pcj8-j9qc)\n\n<details>\n<summary>More information</summary>\n\n#### Details\n##### Impact\nAn unauthenticated remote attacker can initiate a gRPC stream and\npurposefully fragment their payload into millions of tiny (e.g., 1-byte)\nHTTP/2 DATA frames. Even if the total payload volume falls within the\nconfigured connection and stream flow-control windows, each independent\nfragment incurs memory overhead due to internal tracking structures and\nqueue allocation.\n\nRepeated fragmentation massively inflates the heap space consumed by the\nstream. An attacker multiplexing multiple concurrent streams can exhaust\nthe memory bounds of the runtime, forcing a runtime panic or OutOfMemory\ncondition and leading to a remote Denial of Service (DoS).\n\n##### Patches\nThe change to fix this issue is merged in `master` and a patch release,\n1.83.1, has been published that contains this fix.\n\n##### Workarounds\nThis vulnerability is mitigated by implementing receive buffer\ncompaction. Consecutive small data buffers are automatically coalesced\ninto larger buffers from a shared pool once the overhead is perceived to\nbe excessive relative to actual payload data, drastically minimizing\nper-frame memory overheads.\n\nThis behavior is enabled by default. A temporary escape hatch is\nprovided via the environment variable\n`GRPC_GO_EXPERIMENTAL_ENABLE_RECEIVE_BUFFER_COMPACTION=false` to disable\nthe feature if unforeseen issues arise, but it will be removed in a\nfuture release.\n\n#### Severity\n- CVSS Score: 8.7 / 10 (High)\n- Vector String:\n`CVSS:4.0/AV:N/AC:L/AT:N/PR:N/UI:N/VC:N/VI:N/VA:H/SC:N/SI:N/SA:N`\n\n#### References\n-\n[https://github.com/grpc/grpc-go/security/advisories/GHSA-vp52-pcj8-j9qc](https://redirect.github.com/grpc/grpc-go/security/advisories/GHSA-vp52-pcj8-j9qc)\n-\n[https://nvd.nist.gov/vuln/detail/CVE-2026-84304](https://nvd.nist.gov/vuln/detail/CVE-2026-84304)\n-\n[https://github.com/grpc/grpc-go/pull/9331](https://redirect.github.com/grpc/grpc-go/pull/9331)\n-\n[https://github.com/grpc/grpc-go/pull/9333](https://redirect.github.com/grpc/grpc-go/pull/9333)\n-\n[https://github.com/grpc/grpc-go/commit/7354d9c8debb4bcf2225bf429857078de310c176](https://redirect.github.com/grpc/grpc-go/commit/7354d9c8debb4bcf2225bf429857078de310c176)\n-\n[https://github.com/grpc/grpc-go/commit/8cfeca0e1ee5ea0980dcc320e20240fa1079ec77](https://redirect.github.com/grpc/grpc-go/commit/8cfeca0e1ee5ea0980dcc320e20240fa1079ec77)\n-\n[https://github.com/grpc/grpc-go](https://redirect.github.com/grpc/grpc-go)\n-\n[https://github.com/grpc/grpc-go/releases/tag/v1.83.1](https://redirect.github.com/grpc/grpc-go/releases/tag/v1.83.1)\n\nThis data is provided by\n[OSV](https://osv.dev/vulnerability/GHSA-vp52-pcj8-j9qc) and the [GitHub\nAdvisory Database](https://redirect.github.com/github/advisory-database)\n([CC-BY\n4.0](https://redirect.github.com/github/advisory-database/blob/main/LICENSE.md)).\n</details>\n\n---\n\n### Release Notes\n\n<details>\n<summary>grpc/grpc-go (google.golang.org/grpc)</summary>\n\n###\n[`v1.83.1`](https://redirect.github.com/grpc/grpc-go/releases/tag/v1.83.1):\nRelease 1.83.1\n\n[Compare\nSource](https://redirect.github.com/grpc/grpc-go/compare/v1.83.0...v1.83.1)\n\n### Security\n\n- xds/rbac: Fix a bug where nested `Principal` or `Permission` rules\nwith `:scheme` or `grpc-` prefixed header matchers were not rejected,\nwhich could cause DENY rules to fail open.\n([#&#8203;9258](https://redirect.github.com/grpc/grpc-go/issues/9258))\n  - Special Thanks: [@&#8203;nvxbug](https://redirect.github.com/nvxbug)\n- xds/rbac: Fix a bug where the `host` header matcher was not being\nreplaced with `:authority` in nested `Principal` or `Permission` rules.\n([#&#8203;9258](https://redirect.github.com/grpc/grpc-go/issues/9258))\n  - Special Thanks: [@&#8203;nvxbug](https://redirect.github.com/nvxbug)\n- xds/rbac: Fix a bug where a header matcher whose name was not\nlowercase, such as `X-Role`, matched no header, which could cause DENY\nrules to fail open.\n([#&#8203;9332](https://redirect.github.com/grpc/grpc-go/issues/9332))\n- Special Thanks: [@&#8203;alimony](https://redirect.github.com/alimony)\n- xds/rbac: Fix a bug where a `:scheme` or `grpc-` prefixed header\nmatcher was accepted when its name was not lowercase.\n([#&#8203;9332](https://redirect.github.com/grpc/grpc-go/issues/9332))\n- Special Thanks: [@&#8203;alimony](https://redirect.github.com/alimony)\n- xds/rbac: Fix a bug where a `Host` header matcher was not replaced\nwith `:authority`.\n([#&#8203;9332](https://redirect.github.com/grpc/grpc-go/issues/9332))\n- Special Thanks: [@&#8203;alimony](https://redirect.github.com/alimony)\n\n### Performance\n\n- transport: Restrict memory overhead of buffering small data frames.\n([#&#8203;9331](https://redirect.github.com/grpc/grpc-go/issues/9331))\n\n###\n[`v1.83.0`](https://redirect.github.com/grpc/grpc-go/releases/tag/v1.83.0):\nRelease 1.83.0\n\n[Compare\nSource](https://redirect.github.com/grpc/grpc-go/compare/v1.82.2...v1.83.0)\n\n### Security\n\n- server: Stop reading from connections when flooded by HTTP/2 frames to\nmitigate resource exhaustion. The default value for this limit is 100\nframes, excluding DATA and HEADERS, and may be changed by setting\nenvironment variable\n`GRPC_GO_EXPERIMENTAL_CONTROL_BUFFER_THROTTLE_LIMIT`.\n- xds/rbac: Support `Metadata` and `RequestedServerName` permissions\nmatcher fields. If present in a DENY rule, previously these would be\nignored and fail-open.\n- xds/rbac: Fix panic when parsing unsupported fields in\n`NotRule`/`NotId` permissions.\n- xds/rbac: Support the deprecated `source_ip` principal identifier by\ntreating it as equivalent to `direct_remote_ip`.\n- xds: Fix panic when parsing route header matchers configured with\nempty `exact_match`, `prefix_match`, or `suffix_match` strings.\n([#&#8203;9223](https://redirect.github.com/grpc/grpc-go/issues/9223))\n\n### New Features\n\n- xds/googlec2p: Enable DirectPath over Interconnect support for\non-premises clients via the `force-xds` target URI query parameter.\n([#&#8203;9133](https://redirect.github.com/grpc/grpc-go/issues/9133))\n- xds: Enable xDS configuration to control which fields get propagated\nfrom ORCA backend metric reports to LRS load reports.\n([#&#8203;9145](https://redirect.github.com/grpc/grpc-go/issues/9145))\n- authz: Add `OnPolicyUpdate` callback to `FileWatcherOptions` to notify\nwhen an authz policy is loaded or updated.\n([#&#8203;9142](https://redirect.github.com/grpc/grpc-go/issues/9142))\n- Special Thanks: [@&#8203;hnefatl](https://redirect.github.com/hnefatl)\n- xds: Add support for the GCP Authentication HTTP Filter, which\nautomatically fetches and attaches GCP Service Account Identity JWT\ntokens to outgoing RPCs.\n- This feature can be enabled by setting environment variable\n`GRPC_EXPERIMENTAL_XDS_GCP_AUTHENTICATION_FILTER=true`.\n([#&#8203;9119](https://redirect.github.com/grpc/grpc-go/issues/9119))\n- xds: Add support for xDS-based HTTP CONNECT proxies.\n- This feature can be enabled by setting environment variable\n`GRPC_EXPERIMENTAL_XDS_HTTP_CONNECT=true`.\n([#&#8203;9151](https://redirect.github.com/grpc/grpc-go/issues/9151))\n- xds: Add support for `contains_match` in route header matchers.\n([#&#8203;9223](https://redirect.github.com/grpc/grpc-go/issues/9223))\n\n### Bug Fixes\n\n- credentials/alts: Fix panic when processing malformed frames by\nvalidating that the message frame length exceeds the message type field\nsize.\n([#&#8203;9197](https://redirect.github.com/grpc/grpc-go/issues/9197))\n- grpc: Fix compilation on Plan 9 targets (`GOOS=plan9`), broken since\nv1.81.0.\n([#&#8203;9255](https://redirect.github.com/grpc/grpc-go/issues/9255))\n- Special Thanks:\n[@&#8203;Yusufihsangorgel](https://redirect.github.com/Yusufihsangorgel)\n\n###\n[`v1.82.2`](https://redirect.github.com/grpc/grpc-go/releases/tag/v1.82.2):\nRelease 1.82.2\n\n[Compare\nSource](https://redirect.github.com/grpc/grpc-go/compare/v1.82.1...v1.82.2)\n\n### Security\n\n- server: Reject requests missing both `:authority` and `Host` headers\nwith HTTP 400 and status `Internal`.\n([#&#8203;9365](https://redirect.github.com/grpc/grpc-go/pull/9365))\n- Special Thanks:\n[@&#8203;winklemad](https://redirect.github.com/winklemad)\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: (UTC)\n\n- Branch creation\n  - At any time (no schedule defined)\n- Automerge\n  - At any time (no schedule defined)\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0NC40OS4wIiwidXBkYXRlZEluVmVyIjoiNDQuNTcuMyIsInRhcmdldEJyYW5jaCI6Im1haW4iLCJsYWJlbHMiOlsiYXJlYTpzZWN1cml0eSIsImRlcGVuZGVuY2llcyJdfQ==-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2026-09-02T16:29:14Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/11b92ef5b8a51aea55be14a84c9232760a6f5095"
        },
        "date": 1788370415820,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "log_rows_written_rate",
            "value": 99543.54771955963,
            "unit": "rows/sec",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - ClickHouse rows written"
          },
          {
            "name": "logs_produced_rate",
            "value": 100812.65755338164,
            "unit": "rows/sec",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - logs produced"
          },
          {
            "name": "logs_produced_rate",
            "value": 100219.81773174628,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - logs_produced"
          },
          {
            "name": "log_rows_written_rate",
            "value": 99950.35798886552,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - ClickHouse rows written"
          },
          {
            "name": "logs_produced_rate",
            "value": 100580.1226789167,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - logs_produced"
          },
          {
            "name": "log_rows_written_rate",
            "value": 100078.32110988775,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - ClickHouse rows written"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "albertlockett",
            "username": "albertlockett",
            "email": "a.lockett@f5.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "67653eee12a09b7eadce90ea8d40302916ffff8c",
          "message": "fix: inserting new attributes when no previous attributes did exist (#3982)\n\n# Change summary\n\nFixes two related issues related to inserting new attributes when none\ndid previously exist.\n\nThe first issue was that, when using the transform processor, if writing\na program like:\n```\nlogs | set attributes[\"x\"] = \"y\"\n```\nIf some input batch had no existing attribute record batch, a new one\nwould be created but we wouldn't set the schema metadata to indicate\nthat parent_id the column was not delta encoded. So when we decoded that\nto OTLP, we'd treat it as a delta encoded column and the net effect\nwould be that some attributes would end up missing from the result.\n\nThe second issue is somewhat similar - when inserting attributes using\nthe attribute processor we'd create a new attribute record batch, this\ntime with the correct parent_id encoding, but we'd also create a new ID\ncolumn for the root batch and this did not have indicator that said it\nwas not delta encoded.\n\nThis PR \n- corrects the behaviour to add the proper encoding metadata to the\nid/parent_id columns\n- updates tests so that our tests cover regressions agains these kind of\nerrors\n- adds a check in the upsert attributes helper function to ensure it is\npassed an attribute record batch with plain encoded parent_ids.\n\n<!--Replace with a brief summary of the change in this PR-->\n\n## Related issue\n\n<!--We highly recommend correlation of every PR to an issue-->\n\n* Closes #3985\n\n## Validation\n\nUnit tests\n\n<!--How did you confirm your change has the intended effect?-->\n\n## User-facing changes\n\nno\n\n<!--\nDescribe the impact, or write `None`.\nUser-facing changes require a `.chloggen/*.yaml` entry. If no entry is\nneeded,\ninclude `chore` in the PR title. Documentation-only changes are exempt.\n-->\n\n---------\n\nCo-authored-by: Copilot Autofix powered by AI <175728472+Copilot@users.noreply.github.com>",
          "timestamp": "2026-09-03T00:46:22Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/67653eee12a09b7eadce90ea8d40302916ffff8c"
        },
        "date": 1788400425486,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "log_rows_written_rate",
            "value": 99537.61550507709,
            "unit": "rows/sec",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - ClickHouse rows written"
          },
          {
            "name": "logs_produced_rate",
            "value": 100793.76284095453,
            "unit": "rows/sec",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - logs produced"
          },
          {
            "name": "logs_produced_rate",
            "value": 99255.95047834636,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - logs_produced"
          },
          {
            "name": "log_rows_written_rate",
            "value": 100079.82143452045,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - ClickHouse rows written"
          },
          {
            "name": "logs_produced_rate",
            "value": 100343.34482298077,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - logs_produced"
          },
          {
            "name": "log_rows_written_rate",
            "value": 100080.32155272788,
            "unit": "rows/sec",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - ClickHouse rows written"
          }
        ]
      }
    ]
  }
}