window.BENCHMARK_DATA = {
  "lastUpdate": 1787162001865,
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
        "date": 1786673461526,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "clickhouse_cpu_percentage_normalized_avg",
            "value": 49.23614674848247,
            "unit": "%",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - clickhouse CPU"
          },
          {
            "name": "go-collector_cpu_percentage_normalized_avg",
            "value": 88.90347463223925,
            "unit": "%",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - go-collector CPU"
          },
          {
            "name": "clickhouse_ram_mib_max",
            "value": 1230.65234375,
            "unit": "MiB",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - clickhouse RAM"
          },
          {
            "name": "go-collector_ram_mib_max",
            "value": 184.921875,
            "unit": "MiB",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - go-collector RAM"
          },
          {
            "name": "df-engine_cpu_percentage_normalized_avg",
            "value": 45.25406293142949,
            "unit": "%",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - df-engine CPU"
          },
          {
            "name": "clickhouse_cpu_percentage_normalized_avg",
            "value": 49.07707769739969,
            "unit": "%",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - clickhouse CPU"
          },
          {
            "name": "clickhouse_ram_mib_max",
            "value": 1185.3984375,
            "unit": "MiB",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - clickhouse RAM"
          },
          {
            "name": "df-engine_ram_mib_max",
            "value": 54.76953125,
            "unit": "MiB",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - df-engine RAM"
          },
          {
            "name": "test_duration",
            "value": 60.006304,
            "unit": "seconds",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - Test Duration"
          },
          {
            "name": "df-engine_cpu_percentage_normalized_avg",
            "value": 25.703818531859195,
            "unit": "%",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - df-engine CPU"
          },
          {
            "name": "clickhouse_cpu_percentage_normalized_avg",
            "value": 51.1439291334874,
            "unit": "%",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - clickhouse CPU"
          },
          {
            "name": "df-engine_ram_mib_max",
            "value": 53.88671875,
            "unit": "MiB",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - df-engine RAM"
          },
          {
            "name": "clickhouse_ram_mib_max",
            "value": 1191.84375,
            "unit": "MiB",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - clickhouse RAM"
          },
          {
            "name": "test_duration",
            "value": 60.006977,
            "unit": "seconds",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - Test Duration"
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
        "date": 1786730485690,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "clickhouse_cpu_percentage_normalized_avg",
            "value": 49.216111668654186,
            "unit": "%",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - clickhouse CPU"
          },
          {
            "name": "go-collector_cpu_percentage_normalized_avg",
            "value": 86.88587789597348,
            "unit": "%",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - go-collector CPU"
          },
          {
            "name": "go-collector_ram_mib_max",
            "value": 187.59765625,
            "unit": "MiB",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - go-collector RAM"
          },
          {
            "name": "clickhouse_ram_mib_max",
            "value": 1193.8671875,
            "unit": "MiB",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - clickhouse RAM"
          },
          {
            "name": "clickhouse_cpu_percentage_normalized_avg",
            "value": 52.373308163019075,
            "unit": "%",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - clickhouse CPU"
          },
          {
            "name": "df-engine_cpu_percentage_normalized_avg",
            "value": 24.512110343541842,
            "unit": "%",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - df-engine CPU"
          },
          {
            "name": "clickhouse_ram_mib_max",
            "value": 1166.9375,
            "unit": "MiB",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - clickhouse RAM"
          },
          {
            "name": "df-engine_ram_mib_max",
            "value": 55.0703125,
            "unit": "MiB",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - df-engine RAM"
          },
          {
            "name": "test_duration",
            "value": 60.002099,
            "unit": "seconds",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - Test Duration"
          },
          {
            "name": "df-engine_cpu_percentage_normalized_avg",
            "value": 43.22042733241614,
            "unit": "%",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - df-engine CPU"
          },
          {
            "name": "clickhouse_cpu_percentage_normalized_avg",
            "value": 49.17888175392149,
            "unit": "%",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - clickhouse CPU"
          },
          {
            "name": "clickhouse_ram_mib_max",
            "value": 1169.03515625,
            "unit": "MiB",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - clickhouse RAM"
          },
          {
            "name": "df-engine_ram_mib_max",
            "value": 54.3203125,
            "unit": "MiB",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - df-engine RAM"
          },
          {
            "name": "test_duration",
            "value": 60.003834,
            "unit": "seconds",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - Test Duration"
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
        "date": 1786759115394,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "clickhouse_cpu_percentage_normalized_avg",
            "value": 50.37637242014691,
            "unit": "%",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - clickhouse CPU"
          },
          {
            "name": "go-collector_cpu_percentage_normalized_avg",
            "value": 87.00470674681216,
            "unit": "%",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - go-collector CPU"
          },
          {
            "name": "clickhouse_ram_mib_max",
            "value": 1258.64453125,
            "unit": "MiB",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - clickhouse RAM"
          },
          {
            "name": "go-collector_ram_mib_max",
            "value": 182.4140625,
            "unit": "MiB",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - go-collector RAM"
          },
          {
            "name": "df-engine_cpu_percentage_normalized_avg",
            "value": 25.64746895184858,
            "unit": "%",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - df-engine CPU"
          },
          {
            "name": "clickhouse_cpu_percentage_normalized_avg",
            "value": 52.72072198910972,
            "unit": "%",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - clickhouse CPU"
          },
          {
            "name": "clickhouse_ram_mib_max",
            "value": 1215.41015625,
            "unit": "MiB",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - clickhouse RAM"
          },
          {
            "name": "df-engine_ram_mib_max",
            "value": 53.46484375,
            "unit": "MiB",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - df-engine RAM"
          },
          {
            "name": "test_duration",
            "value": 60.001874,
            "unit": "seconds",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - Test Duration"
          },
          {
            "name": "df-engine_cpu_percentage_normalized_avg",
            "value": 45.57007330361482,
            "unit": "%",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - df-engine CPU"
          },
          {
            "name": "clickhouse_cpu_percentage_normalized_avg",
            "value": 48.90010340509271,
            "unit": "%",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - clickhouse CPU"
          },
          {
            "name": "clickhouse_ram_mib_max",
            "value": 1160.84765625,
            "unit": "MiB",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - clickhouse RAM"
          },
          {
            "name": "df-engine_ram_mib_max",
            "value": 52.9765625,
            "unit": "MiB",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - df-engine RAM"
          },
          {
            "name": "test_duration",
            "value": 60.001969,
            "unit": "seconds",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - Test Duration"
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
        "date": 1786816076170,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "clickhouse_cpu_percentage_normalized_avg",
            "value": 48.683736018209665,
            "unit": "%",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - clickhouse CPU"
          },
          {
            "name": "go-collector_cpu_percentage_normalized_avg",
            "value": 89.97064775352892,
            "unit": "%",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - go-collector CPU"
          },
          {
            "name": "clickhouse_ram_mib_max",
            "value": 1209.91796875,
            "unit": "MiB",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - clickhouse RAM"
          },
          {
            "name": "go-collector_ram_mib_max",
            "value": 186.37109375,
            "unit": "MiB",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - go-collector RAM"
          },
          {
            "name": "df-engine_cpu_percentage_normalized_avg",
            "value": 44.43755163380843,
            "unit": "%",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - df-engine CPU"
          },
          {
            "name": "clickhouse_cpu_percentage_normalized_avg",
            "value": 49.75915122433234,
            "unit": "%",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - clickhouse CPU"
          },
          {
            "name": "clickhouse_ram_mib_max",
            "value": 1138.12109375,
            "unit": "MiB",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - clickhouse RAM"
          },
          {
            "name": "df-engine_ram_mib_max",
            "value": 56.6171875,
            "unit": "MiB",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - df-engine RAM"
          },
          {
            "name": "test_duration",
            "value": 60.003777,
            "unit": "seconds",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - Test Duration"
          },
          {
            "name": "clickhouse_cpu_percentage_normalized_avg",
            "value": 51.43642166798317,
            "unit": "%",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - clickhouse CPU"
          },
          {
            "name": "df-engine_cpu_percentage_normalized_avg",
            "value": 25.785838522324074,
            "unit": "%",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - df-engine CPU"
          },
          {
            "name": "clickhouse_ram_mib_max",
            "value": 1213.5625,
            "unit": "MiB",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - clickhouse RAM"
          },
          {
            "name": "df-engine_ram_mib_max",
            "value": 50.3515625,
            "unit": "MiB",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - df-engine RAM"
          },
          {
            "name": "test_duration",
            "value": 60.005697,
            "unit": "seconds",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - Test Duration"
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
        "date": 1786845494260,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "clickhouse_cpu_percentage_normalized_avg",
            "value": 47.413743293442025,
            "unit": "%",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - clickhouse CPU"
          },
          {
            "name": "go-collector_cpu_percentage_normalized_avg",
            "value": 89.5288661692355,
            "unit": "%",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - go-collector CPU"
          },
          {
            "name": "clickhouse_ram_mib_max",
            "value": 1232.9765625,
            "unit": "MiB",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - clickhouse RAM"
          },
          {
            "name": "go-collector_ram_mib_max",
            "value": 183.03515625,
            "unit": "MiB",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - go-collector RAM"
          },
          {
            "name": "df-engine_cpu_percentage_normalized_avg",
            "value": 45.13798301723538,
            "unit": "%",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - df-engine CPU"
          },
          {
            "name": "clickhouse_cpu_percentage_normalized_avg",
            "value": 48.78899206228843,
            "unit": "%",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - clickhouse CPU"
          },
          {
            "name": "clickhouse_ram_mib_max",
            "value": 1150.42578125,
            "unit": "MiB",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - clickhouse RAM"
          },
          {
            "name": "df-engine_ram_mib_max",
            "value": 52.00390625,
            "unit": "MiB",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - df-engine RAM"
          },
          {
            "name": "test_duration",
            "value": 60.003839,
            "unit": "seconds",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - Test Duration"
          },
          {
            "name": "df-engine_cpu_percentage_normalized_avg",
            "value": 25.643229450921684,
            "unit": "%",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - df-engine CPU"
          },
          {
            "name": "clickhouse_cpu_percentage_normalized_avg",
            "value": 49.44349604319595,
            "unit": "%",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - clickhouse CPU"
          },
          {
            "name": "clickhouse_ram_mib_max",
            "value": 1196.71484375,
            "unit": "MiB",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - clickhouse RAM"
          },
          {
            "name": "df-engine_ram_mib_max",
            "value": 52.828125,
            "unit": "MiB",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - df-engine RAM"
          },
          {
            "name": "test_duration",
            "value": 60.002005,
            "unit": "seconds",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - Test Duration"
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
        "date": 1786902487530,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "clickhouse_cpu_percentage_normalized_avg",
            "value": 49.78158481155384,
            "unit": "%",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - clickhouse CPU"
          },
          {
            "name": "go-collector_cpu_percentage_normalized_avg",
            "value": 89.11358413890258,
            "unit": "%",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - go-collector CPU"
          },
          {
            "name": "clickhouse_ram_mib_max",
            "value": 1217.72265625,
            "unit": "MiB",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - clickhouse RAM"
          },
          {
            "name": "go-collector_ram_mib_max",
            "value": 197.8828125,
            "unit": "MiB",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - go-collector RAM"
          },
          {
            "name": "df-engine_cpu_percentage_normalized_avg",
            "value": 25.460453102078716,
            "unit": "%",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - df-engine CPU"
          },
          {
            "name": "clickhouse_cpu_percentage_normalized_avg",
            "value": 49.63757609903523,
            "unit": "%",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - clickhouse CPU"
          },
          {
            "name": "clickhouse_ram_mib_max",
            "value": 1175.33984375,
            "unit": "MiB",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - clickhouse RAM"
          },
          {
            "name": "df-engine_ram_mib_max",
            "value": 51.5078125,
            "unit": "MiB",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - df-engine RAM"
          },
          {
            "name": "test_duration",
            "value": 60.006922,
            "unit": "seconds",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - Test Duration"
          },
          {
            "name": "df-engine_cpu_percentage_normalized_avg",
            "value": 45.51534634098881,
            "unit": "%",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - df-engine CPU"
          },
          {
            "name": "clickhouse_cpu_percentage_normalized_avg",
            "value": 49.05414128781199,
            "unit": "%",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - clickhouse CPU"
          },
          {
            "name": "clickhouse_ram_mib_max",
            "value": 1156.94921875,
            "unit": "MiB",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - clickhouse RAM"
          },
          {
            "name": "df-engine_ram_mib_max",
            "value": 52.16015625,
            "unit": "MiB",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - df-engine RAM"
          },
          {
            "name": "test_duration",
            "value": 60.003771,
            "unit": "seconds",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - Test Duration"
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
        "date": 1786931871553,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "clickhouse_cpu_percentage_normalized_avg",
            "value": 50.4312625895341,
            "unit": "%",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - clickhouse CPU"
          },
          {
            "name": "go-collector_cpu_percentage_normalized_avg",
            "value": 88.81567837311572,
            "unit": "%",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - go-collector CPU"
          },
          {
            "name": "clickhouse_ram_mib_max",
            "value": 1256.6953125,
            "unit": "MiB",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - clickhouse RAM"
          },
          {
            "name": "go-collector_ram_mib_max",
            "value": 177.0625,
            "unit": "MiB",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - go-collector RAM"
          },
          {
            "name": "clickhouse_cpu_percentage_normalized_avg",
            "value": 49.585087098789074,
            "unit": "%",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - clickhouse CPU"
          },
          {
            "name": "df-engine_cpu_percentage_normalized_avg",
            "value": 24.83731306297865,
            "unit": "%",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - df-engine CPU"
          },
          {
            "name": "df-engine_ram_mib_max",
            "value": 53.2265625,
            "unit": "MiB",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - df-engine RAM"
          },
          {
            "name": "clickhouse_ram_mib_max",
            "value": 1251.5,
            "unit": "MiB",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - clickhouse RAM"
          },
          {
            "name": "test_duration",
            "value": 60.006766,
            "unit": "seconds",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - Test Duration"
          },
          {
            "name": "df-engine_cpu_percentage_normalized_avg",
            "value": 44.44119082160014,
            "unit": "%",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - df-engine CPU"
          },
          {
            "name": "clickhouse_cpu_percentage_normalized_avg",
            "value": 48.84783852623003,
            "unit": "%",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - clickhouse CPU"
          },
          {
            "name": "clickhouse_ram_mib_max",
            "value": 1139.484375,
            "unit": "MiB",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - clickhouse RAM"
          },
          {
            "name": "df-engine_ram_mib_max",
            "value": 54.97265625,
            "unit": "MiB",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - df-engine RAM"
          },
          {
            "name": "test_duration",
            "value": 60.003704,
            "unit": "seconds",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - Test Duration"
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
        "date": 1786988992403,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "clickhouse_cpu_percentage_normalized_avg",
            "value": 48.27271760484478,
            "unit": "%",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - clickhouse CPU"
          },
          {
            "name": "go-collector_cpu_percentage_normalized_avg",
            "value": 88.79716839280648,
            "unit": "%",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - go-collector CPU"
          },
          {
            "name": "clickhouse_ram_mib_max",
            "value": 1256.60546875,
            "unit": "MiB",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - clickhouse RAM"
          },
          {
            "name": "go-collector_ram_mib_max",
            "value": 181.4921875,
            "unit": "MiB",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - go-collector RAM"
          },
          {
            "name": "df-engine_cpu_percentage_normalized_avg",
            "value": 25.795245238817166,
            "unit": "%",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - df-engine CPU"
          },
          {
            "name": "clickhouse_cpu_percentage_normalized_avg",
            "value": 50.851908227799136,
            "unit": "%",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - clickhouse CPU"
          },
          {
            "name": "clickhouse_ram_mib_max",
            "value": 1170.83203125,
            "unit": "MiB",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - clickhouse RAM"
          },
          {
            "name": "df-engine_ram_mib_max",
            "value": 51.40625,
            "unit": "MiB",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - df-engine RAM"
          },
          {
            "name": "test_duration",
            "value": 60.002196,
            "unit": "seconds",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - Test Duration"
          },
          {
            "name": "df-engine_cpu_percentage_normalized_avg",
            "value": 45.31258424659011,
            "unit": "%",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - df-engine CPU"
          },
          {
            "name": "clickhouse_cpu_percentage_normalized_avg",
            "value": 47.81980906540513,
            "unit": "%",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - clickhouse CPU"
          },
          {
            "name": "clickhouse_ram_mib_max",
            "value": 1133.5703125,
            "unit": "MiB",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - clickhouse RAM"
          },
          {
            "name": "df-engine_ram_mib_max",
            "value": 51.13671875,
            "unit": "MiB",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - df-engine RAM"
          },
          {
            "name": "test_duration",
            "value": 60.003739,
            "unit": "seconds",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - Test Duration"
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
        "date": 1787019111423,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "clickhouse_cpu_percentage_normalized_avg",
            "value": 47.41230953875578,
            "unit": "%",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - clickhouse CPU"
          },
          {
            "name": "go-collector_cpu_percentage_normalized_avg",
            "value": 89.8221609814651,
            "unit": "%",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - go-collector CPU"
          },
          {
            "name": "clickhouse_ram_mib_max",
            "value": 1280.8671875,
            "unit": "MiB",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - clickhouse RAM"
          },
          {
            "name": "go-collector_ram_mib_max",
            "value": 186.453125,
            "unit": "MiB",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - go-collector RAM"
          },
          {
            "name": "df-engine_cpu_percentage_normalized_avg",
            "value": 25.451015162181445,
            "unit": "%",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - df-engine CPU"
          },
          {
            "name": "clickhouse_cpu_percentage_normalized_avg",
            "value": 49.01182168388771,
            "unit": "%",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - clickhouse CPU"
          },
          {
            "name": "df-engine_ram_mib_max",
            "value": 52.62109375,
            "unit": "MiB",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - df-engine RAM"
          },
          {
            "name": "clickhouse_ram_mib_max",
            "value": 1204.375,
            "unit": "MiB",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - clickhouse RAM"
          },
          {
            "name": "test_duration",
            "value": 60.002194,
            "unit": "seconds",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - Test Duration"
          },
          {
            "name": "df-engine_cpu_percentage_normalized_avg",
            "value": 45.2358024098268,
            "unit": "%",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - df-engine CPU"
          },
          {
            "name": "clickhouse_cpu_percentage_normalized_avg",
            "value": 50.224186180948706,
            "unit": "%",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - clickhouse CPU"
          },
          {
            "name": "clickhouse_ram_mib_max",
            "value": 1220.62109375,
            "unit": "MiB",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - clickhouse RAM"
          },
          {
            "name": "df-engine_ram_mib_max",
            "value": 50.60546875,
            "unit": "MiB",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - df-engine RAM"
          },
          {
            "name": "test_duration",
            "value": 60.003637,
            "unit": "seconds",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - Test Duration"
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
        "date": 1787090398145,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "clickhouse_cpu_percentage_normalized_avg",
            "value": 47.88782978966325,
            "unit": "%",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - clickhouse CPU"
          },
          {
            "name": "go-collector_cpu_percentage_normalized_avg",
            "value": 88.1900210892835,
            "unit": "%",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - go-collector CPU"
          },
          {
            "name": "clickhouse_ram_mib_max",
            "value": 1245.51953125,
            "unit": "MiB",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - clickhouse RAM"
          },
          {
            "name": "go-collector_ram_mib_max",
            "value": 191.84375,
            "unit": "MiB",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - go-collector RAM"
          },
          {
            "name": "df-engine_cpu_percentage_normalized_avg",
            "value": 44.45457531064624,
            "unit": "%",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - df-engine CPU"
          },
          {
            "name": "clickhouse_cpu_percentage_normalized_avg",
            "value": 49.87989618111646,
            "unit": "%",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - clickhouse CPU"
          },
          {
            "name": "df-engine_ram_mib_max",
            "value": 50.47265625,
            "unit": "MiB",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - df-engine RAM"
          },
          {
            "name": "clickhouse_ram_mib_max",
            "value": 1210.8359375,
            "unit": "MiB",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - clickhouse RAM"
          },
          {
            "name": "test_duration",
            "value": 60.003533,
            "unit": "seconds",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - Test Duration"
          },
          {
            "name": "df-engine_cpu_percentage_normalized_avg",
            "value": 25.17829253098792,
            "unit": "%",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - df-engine CPU"
          },
          {
            "name": "clickhouse_cpu_percentage_normalized_avg",
            "value": 50.55381891862151,
            "unit": "%",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - clickhouse CPU"
          },
          {
            "name": "clickhouse_ram_mib_max",
            "value": 1192.12890625,
            "unit": "MiB",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - clickhouse RAM"
          },
          {
            "name": "df-engine_ram_mib_max",
            "value": 52.53515625,
            "unit": "MiB",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - df-engine RAM"
          },
          {
            "name": "test_duration",
            "value": 60.001861,
            "unit": "seconds",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - Test Duration"
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
        "date": 1787107223777,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "clickhouse_cpu_percentage_normalized_avg",
            "value": 47.960734670253856,
            "unit": "%",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - clickhouse CPU"
          },
          {
            "name": "go-collector_cpu_percentage_normalized_avg",
            "value": 88.74218772899744,
            "unit": "%",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - go-collector CPU"
          },
          {
            "name": "clickhouse_ram_mib_max",
            "value": 1207.8046875,
            "unit": "MiB",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - clickhouse RAM"
          },
          {
            "name": "go-collector_ram_mib_max",
            "value": 191.6953125,
            "unit": "MiB",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - go-collector RAM"
          },
          {
            "name": "df-engine_cpu_percentage_normalized_avg",
            "value": 25.630815993712535,
            "unit": "%",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - df-engine CPU"
          },
          {
            "name": "clickhouse_cpu_percentage_normalized_avg",
            "value": 51.59787473134843,
            "unit": "%",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - clickhouse CPU"
          },
          {
            "name": "clickhouse_ram_mib_max",
            "value": 1180.26953125,
            "unit": "MiB",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - clickhouse RAM"
          },
          {
            "name": "df-engine_ram_mib_max",
            "value": 50.7265625,
            "unit": "MiB",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - df-engine RAM"
          },
          {
            "name": "test_duration",
            "value": 60.015674,
            "unit": "seconds",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - Test Duration"
          },
          {
            "name": "df-engine_cpu_percentage_normalized_avg",
            "value": 44.83910066558121,
            "unit": "%",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - df-engine CPU"
          },
          {
            "name": "clickhouse_cpu_percentage_normalized_avg",
            "value": 50.25437625029333,
            "unit": "%",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - clickhouse CPU"
          },
          {
            "name": "clickhouse_ram_mib_max",
            "value": 1140.5625,
            "unit": "MiB",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - clickhouse RAM"
          },
          {
            "name": "df-engine_ram_mib_max",
            "value": 49.06640625,
            "unit": "MiB",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - df-engine RAM"
          },
          {
            "name": "test_duration",
            "value": 60.003149,
            "unit": "seconds",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - Test Duration"
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
        "date": 1787162000515,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "clickhouse_cpu_percentage_normalized_avg",
            "value": 48.561085749418154,
            "unit": "%",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - clickhouse CPU"
          },
          {
            "name": "go-collector_cpu_percentage_normalized_avg",
            "value": 89.59674049574626,
            "unit": "%",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - go-collector CPU"
          },
          {
            "name": "clickhouse_ram_mib_max",
            "value": 1221.44140625,
            "unit": "MiB",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - clickhouse RAM"
          },
          {
            "name": "go-collector_ram_mib_max",
            "value": 199.515625,
            "unit": "MiB",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - go-collector RAM"
          },
          {
            "name": "df-engine_cpu_percentage_normalized_avg",
            "value": 45.25538346120069,
            "unit": "%",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - df-engine CPU"
          },
          {
            "name": "clickhouse_cpu_percentage_normalized_avg",
            "value": 48.128061281917155,
            "unit": "%",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - clickhouse CPU"
          },
          {
            "name": "clickhouse_ram_mib_max",
            "value": 1220.640625,
            "unit": "MiB",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - clickhouse RAM"
          },
          {
            "name": "df-engine_ram_mib_max",
            "value": 58.04296875,
            "unit": "MiB",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - df-engine RAM"
          },
          {
            "name": "test_duration",
            "value": 60.003847,
            "unit": "seconds",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - Test Duration"
          },
          {
            "name": "df-engine_cpu_percentage_normalized_avg",
            "value": 25.33831071981342,
            "unit": "%",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - df-engine CPU"
          },
          {
            "name": "clickhouse_cpu_percentage_normalized_avg",
            "value": 52.645869601250425,
            "unit": "%",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - clickhouse CPU"
          },
          {
            "name": "clickhouse_ram_mib_max",
            "value": 1250.11328125,
            "unit": "MiB",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - clickhouse RAM"
          },
          {
            "name": "df-engine_ram_mib_max",
            "value": 55.3359375,
            "unit": "MiB",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - df-engine RAM"
          },
          {
            "name": "test_duration",
            "value": 60.001967,
            "unit": "seconds",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - Test Duration"
          }
        ]
      }
    ]
  }
}