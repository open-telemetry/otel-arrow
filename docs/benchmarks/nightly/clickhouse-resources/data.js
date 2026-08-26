window.BENCHMARK_DATA = {
  "lastUpdate": 1787708123926,
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
        "date": 1787191106097,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "clickhouse_cpu_percentage_normalized_avg",
            "value": 46.95099199357758,
            "unit": "%",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - clickhouse CPU"
          },
          {
            "name": "go-collector_cpu_percentage_normalized_avg",
            "value": 89.29235090135373,
            "unit": "%",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - go-collector CPU"
          },
          {
            "name": "go-collector_ram_mib_max",
            "value": 183.77734375,
            "unit": "MiB",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - go-collector RAM"
          },
          {
            "name": "clickhouse_ram_mib_max",
            "value": 1249.32421875,
            "unit": "MiB",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - clickhouse RAM"
          },
          {
            "name": "df-engine_cpu_percentage_normalized_avg",
            "value": 44.34573044443184,
            "unit": "%",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - df-engine CPU"
          },
          {
            "name": "clickhouse_cpu_percentage_normalized_avg",
            "value": 50.693345343883855,
            "unit": "%",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - clickhouse CPU"
          },
          {
            "name": "clickhouse_ram_mib_max",
            "value": 1210.515625,
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
            "value": 60.003162,
            "unit": "seconds",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - Test Duration"
          },
          {
            "name": "df-engine_cpu_percentage_normalized_avg",
            "value": 24.780918798230157,
            "unit": "%",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - df-engine CPU"
          },
          {
            "name": "clickhouse_cpu_percentage_normalized_avg",
            "value": 51.00333681814415,
            "unit": "%",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - clickhouse CPU"
          },
          {
            "name": "df-engine_ram_mib_max",
            "value": 52.046875,
            "unit": "MiB",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - df-engine RAM"
          },
          {
            "name": "clickhouse_ram_mib_max",
            "value": 1245.53125,
            "unit": "MiB",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - clickhouse RAM"
          },
          {
            "name": "test_duration",
            "value": 60.007045,
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
          "id": "6c815bd0bdf6c5e023daf7987e17ca5b4dda565c",
          "message": "chore(clippy): Resolve clippy 1.98 errors in main (#3835)\n\n# Chore Summary\n\nAddress clippy error on unrelated PR:\nhttps://github.com/open-telemetry/otel-arrow/actions/runs/32402148082/job/96532892549?pr=3834\n\n```text\nerror: draining all elements of a collection into a new collection of the same type\n    --> crates/core-nodes/src/processors/batch_processor/mod.rs:1485:22\n     |\n1485 |             pending: self.pending.drain(..).collect(),\n     |                      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ help: use `mem::take` to avoid creating a new allocation: `std::mem::take(&mut self.pending)`\n     |\n     = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.98.0/index.html#drain_collect\n     = note: `-D clippy::drain-collect` implied by `-D clippy::perf`\n     = help: to override `-D clippy::perf` add `#[allow(clippy::drain_collect)]`\n\nerror: draining all elements of a collection into a new collection of the same type\n    --> crates/core-nodes/src/processors/batch_processor/mod.rs:1486:22\n     |\n1486 |             context: self.context.drain(..).collect(),\n     |                      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ help: use `mem::take` to avoid creating a new allocation: `std::mem::take(&mut self.context)`\n     |\n     = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.98.0/index.html#drain_collect\n\nerror: the `Err`-variant returned from this function is very large\n   --> crates/core-nodes/src/processors/fanout_processor/mod.rs:562:10\n    |\n562 |     ) -> Result<DeadlineVec, TypedError<OtapPdata>> {\n    |          ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ the `Err`-variant is at least 152 bytes\n    |\n    = help: try reducing the size of `otap_df_engine::error::TypedError<otap_df_otap::pdata::OtapPdata>`, for example by boxing large elements or replacing it with `Box<otap_df_engine::error::TypedError<otap_df_otap::pdata::OtapPdata>>`\n    = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.98.0/index.html#result_large_err\n    = note: `-D clippy::result-large-err` implied by `-D clippy::perf`\n    = help: to override `-D clippy::perf` add `#[allow(clippy::result_large_err)]`\n```\n\n## Related issue\n\n<!-- Link the related issue if one exists. -->",
          "timestamp": "2026-08-20T19:42:52Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/6c815bd0bdf6c5e023daf7987e17ca5b4dda565c"
        },
        "date": 1787257990690,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "clickhouse_cpu_percentage_normalized_avg",
            "value": 49.04833058461453,
            "unit": "%",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - clickhouse CPU"
          },
          {
            "name": "go-collector_cpu_percentage_normalized_avg",
            "value": 87.53471900713544,
            "unit": "%",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - go-collector CPU"
          },
          {
            "name": "clickhouse_ram_mib_max",
            "value": 1236.29296875,
            "unit": "MiB",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - clickhouse RAM"
          },
          {
            "name": "go-collector_ram_mib_max",
            "value": 209.08203125,
            "unit": "MiB",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - go-collector RAM"
          },
          {
            "name": "df-engine_cpu_percentage_normalized_avg",
            "value": 25.046636999551474,
            "unit": "%",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - df-engine CPU"
          },
          {
            "name": "clickhouse_cpu_percentage_normalized_avg",
            "value": 50.537974229472674,
            "unit": "%",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - clickhouse CPU"
          },
          {
            "name": "clickhouse_ram_mib_max",
            "value": 1185.23828125,
            "unit": "MiB",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - clickhouse RAM"
          },
          {
            "name": "df-engine_ram_mib_max",
            "value": 54.0078125,
            "unit": "MiB",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - df-engine RAM"
          },
          {
            "name": "test_duration",
            "value": 60.012337,
            "unit": "seconds",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - Test Duration"
          },
          {
            "name": "df-engine_cpu_percentage_normalized_avg",
            "value": 45.122333592719244,
            "unit": "%",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - df-engine CPU"
          },
          {
            "name": "clickhouse_cpu_percentage_normalized_avg",
            "value": 50.22665579214406,
            "unit": "%",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - clickhouse CPU"
          },
          {
            "name": "clickhouse_ram_mib_max",
            "value": 1156.5234375,
            "unit": "MiB",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - clickhouse RAM"
          },
          {
            "name": "df-engine_ram_mib_max",
            "value": 55.86328125,
            "unit": "MiB",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - df-engine RAM"
          },
          {
            "name": "test_duration",
            "value": 60.003197,
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
          "id": "26b710aa0c6e5900e94523992c018104db6cfe24",
          "message": "chore(engine): add local retained-work accounting (#3756)\n\n# Change Summary\n\nAdd a runtime-local retained-work account and non-`Send` ownership\nticket.\n\nThe account tracks known retained bytes and unknown-size items. Tickets\nrefund their charge exactly once on explicit completion. Dropping an\nunresolved ticket also refunds the charge and records abandonment.\n\nChecked arithmetic reports overflow and underflow as accounting\ncorruption.\n\nThis PR does not add runtime wiring, attribution, metrics export,\nconfiguration, enforcement, escrow, or production charge sites.\n\n## Background\n\nThe retained-work pilot needs a runtime-local accounting primitive\nbefore scope wiring, metrics, or processor integration can be added.\n\n## What issue does this PR close?\n\n* Part of #3272\n\n## How are these changes tested?\n\n- `cargo check -p otap-df-engine`\n- `cargo test -p otap-df-engine retained_work::tests`\n- `cargo test -p otap-df-engine --doc`\n- `cargo clippy -p otap-df-engine --all-targets -- -D warnings`\n- `cargo xtask check`\n- `python3 tools/sanitycheck.py`\n- `git diff --check`\n\n## Are there any user-facing changes?\n\nNo. This PR adds an internal accounting primitive without changing\nruntime behavior, configuration, or exported telemetry.\n\n  ### Changelog\n\n  * [ ] Added a `.chloggen/*.yaml` entry\n  * [x] This PR is a `chore` (indicated in title)\n  * [ ] This is a documentation-only PR.",
          "timestamp": "2026-08-21T00:33:27Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/26b710aa0c6e5900e94523992c018104db6cfe24"
        },
        "date": 1787292495589,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "clickhouse_cpu_percentage_normalized_avg",
            "value": 49.90015831818921,
            "unit": "%",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - clickhouse CPU"
          },
          {
            "name": "go-collector_cpu_percentage_normalized_avg",
            "value": 89.26868635672078,
            "unit": "%",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - go-collector CPU"
          },
          {
            "name": "clickhouse_ram_mib_max",
            "value": 1243.1015625,
            "unit": "MiB",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - clickhouse RAM"
          },
          {
            "name": "go-collector_ram_mib_max",
            "value": 185.6640625,
            "unit": "MiB",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - go-collector RAM"
          },
          {
            "name": "df-engine_cpu_percentage_normalized_avg",
            "value": 25.02172380319208,
            "unit": "%",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - df-engine CPU"
          },
          {
            "name": "clickhouse_cpu_percentage_normalized_avg",
            "value": 49.06376756527753,
            "unit": "%",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - clickhouse CPU"
          },
          {
            "name": "clickhouse_ram_mib_max",
            "value": 1192.625,
            "unit": "MiB",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - clickhouse RAM"
          },
          {
            "name": "df-engine_ram_mib_max",
            "value": 51.12109375,
            "unit": "MiB",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - df-engine RAM"
          },
          {
            "name": "test_duration",
            "value": 60.001979,
            "unit": "seconds",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - Test Duration"
          },
          {
            "name": "df-engine_cpu_percentage_normalized_avg",
            "value": 44.90870574737737,
            "unit": "%",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - df-engine CPU"
          },
          {
            "name": "clickhouse_cpu_percentage_normalized_avg",
            "value": 50.16092664068942,
            "unit": "%",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - clickhouse CPU"
          },
          {
            "name": "clickhouse_ram_mib_max",
            "value": 1187.54296875,
            "unit": "MiB",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - clickhouse RAM"
          },
          {
            "name": "df-engine_ram_mib_max",
            "value": 52.09765625,
            "unit": "MiB",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - df-engine RAM"
          },
          {
            "name": "test_duration",
            "value": 60.005289,
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
          "id": "845e34cdd9da76b9ff3b1fb312fefcbe63678ddb",
          "message": "chore(repo): Update CODEOWNERS to specify required approvals for core engine crates (#3851)\n\n# Chore Summary\n\nPer offline discussion among maintainers, wanted to clarify some\nadditional CODEOWNER policies as contribution trends increase to help\nensure proper visibility to core engine crates.\n\n## Related issue\n\nN/A\n\n---------\n\nCo-authored-by: Copilot Autofix powered by AI <175728472+Copilot@users.noreply.github.com>",
          "timestamp": "2026-08-21T16:29:22Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/845e34cdd9da76b9ff3b1fb312fefcbe63678ddb"
        },
        "date": 1787334717423,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "clickhouse_cpu_percentage_normalized_avg",
            "value": 48.05112969521273,
            "unit": "%",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - clickhouse CPU"
          },
          {
            "name": "go-collector_cpu_percentage_normalized_avg",
            "value": 88.89118873838133,
            "unit": "%",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - go-collector CPU"
          },
          {
            "name": "clickhouse_ram_mib_max",
            "value": 1228.95703125,
            "unit": "MiB",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - clickhouse RAM"
          },
          {
            "name": "go-collector_ram_mib_max",
            "value": 181.08203125,
            "unit": "MiB",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - go-collector RAM"
          },
          {
            "name": "df-engine_cpu_percentage_normalized_avg",
            "value": 25.539575575946227,
            "unit": "%",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - df-engine CPU"
          },
          {
            "name": "clickhouse_cpu_percentage_normalized_avg",
            "value": 49.4984000464772,
            "unit": "%",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - clickhouse CPU"
          },
          {
            "name": "clickhouse_ram_mib_max",
            "value": 1167.328125,
            "unit": "MiB",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - clickhouse RAM"
          },
          {
            "name": "df-engine_ram_mib_max",
            "value": 56.3984375,
            "unit": "MiB",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - df-engine RAM"
          },
          {
            "name": "test_duration",
            "value": 60.001843,
            "unit": "seconds",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - Test Duration"
          },
          {
            "name": "clickhouse_cpu_percentage_normalized_avg",
            "value": 49.31515024257486,
            "unit": "%",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - clickhouse CPU"
          },
          {
            "name": "df-engine_cpu_percentage_normalized_avg",
            "value": 45.41227355381292,
            "unit": "%",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - df-engine CPU"
          },
          {
            "name": "clickhouse_ram_mib_max",
            "value": 1138.8125,
            "unit": "MiB",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - clickhouse RAM"
          },
          {
            "name": "df-engine_ram_mib_max",
            "value": 51.3125,
            "unit": "MiB",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - df-engine RAM"
          },
          {
            "name": "test_duration",
            "value": 60.005082,
            "unit": "seconds",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - Test Duration"
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
        "date": 1787362479399,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "clickhouse_cpu_percentage_normalized_avg",
            "value": 49.32324781308154,
            "unit": "%",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - clickhouse CPU"
          },
          {
            "name": "go-collector_cpu_percentage_normalized_avg",
            "value": 87.31433184281968,
            "unit": "%",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - go-collector CPU"
          },
          {
            "name": "clickhouse_ram_mib_max",
            "value": 1279.12890625,
            "unit": "MiB",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - clickhouse RAM"
          },
          {
            "name": "go-collector_ram_mib_max",
            "value": 193.40625,
            "unit": "MiB",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - go-collector RAM"
          },
          {
            "name": "df-engine_cpu_percentage_normalized_avg",
            "value": 25.839697922928657,
            "unit": "%",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - df-engine CPU"
          },
          {
            "name": "clickhouse_cpu_percentage_normalized_avg",
            "value": 49.851655404567666,
            "unit": "%",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - clickhouse CPU"
          },
          {
            "name": "clickhouse_ram_mib_max",
            "value": 1177.83203125,
            "unit": "MiB",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - clickhouse RAM"
          },
          {
            "name": "df-engine_ram_mib_max",
            "value": 51.61328125,
            "unit": "MiB",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - df-engine RAM"
          },
          {
            "name": "test_duration",
            "value": 60.00715,
            "unit": "seconds",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - Test Duration"
          },
          {
            "name": "df-engine_cpu_percentage_normalized_avg",
            "value": 45.48642088246655,
            "unit": "%",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - df-engine CPU"
          },
          {
            "name": "clickhouse_cpu_percentage_normalized_avg",
            "value": 49.460348995429875,
            "unit": "%",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - clickhouse CPU"
          },
          {
            "name": "clickhouse_ram_mib_max",
            "value": 1211.25390625,
            "unit": "MiB",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - clickhouse RAM"
          },
          {
            "name": "df-engine_ram_mib_max",
            "value": 55.86328125,
            "unit": "MiB",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - df-engine RAM"
          },
          {
            "name": "test_duration",
            "value": 60.003421,
            "unit": "seconds",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - Test Duration"
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
        "date": 1787419465885,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "clickhouse_cpu_percentage_normalized_avg",
            "value": 48.91939760546548,
            "unit": "%",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - clickhouse CPU"
          },
          {
            "name": "go-collector_cpu_percentage_normalized_avg",
            "value": 88.61965912026155,
            "unit": "%",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - go-collector CPU"
          },
          {
            "name": "clickhouse_ram_mib_max",
            "value": 1268.3125,
            "unit": "MiB",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - clickhouse RAM"
          },
          {
            "name": "go-collector_ram_mib_max",
            "value": 181.41015625,
            "unit": "MiB",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - go-collector RAM"
          },
          {
            "name": "df-engine_cpu_percentage_normalized_avg",
            "value": 45.076070607500235,
            "unit": "%",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - df-engine CPU"
          },
          {
            "name": "clickhouse_cpu_percentage_normalized_avg",
            "value": 50.81486652885199,
            "unit": "%",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - clickhouse CPU"
          },
          {
            "name": "clickhouse_ram_mib_max",
            "value": 1163.52734375,
            "unit": "MiB",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - clickhouse RAM"
          },
          {
            "name": "df-engine_ram_mib_max",
            "value": 51.62890625,
            "unit": "MiB",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - df-engine RAM"
          },
          {
            "name": "test_duration",
            "value": 60.004032,
            "unit": "seconds",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - Test Duration"
          },
          {
            "name": "df-engine_cpu_percentage_normalized_avg",
            "value": 25.801781446803794,
            "unit": "%",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - df-engine CPU"
          },
          {
            "name": "clickhouse_cpu_percentage_normalized_avg",
            "value": 50.642741488299706,
            "unit": "%",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - clickhouse CPU"
          },
          {
            "name": "clickhouse_ram_mib_max",
            "value": 1247.390625,
            "unit": "MiB",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - clickhouse RAM"
          },
          {
            "name": "df-engine_ram_mib_max",
            "value": 51.65625,
            "unit": "MiB",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - df-engine RAM"
          },
          {
            "name": "test_duration",
            "value": 60.002091,
            "unit": "seconds",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - Test Duration"
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
        "date": 1787448898383,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "clickhouse_cpu_percentage_normalized_avg",
            "value": 50.38998891699716,
            "unit": "%",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - clickhouse CPU"
          },
          {
            "name": "go-collector_cpu_percentage_normalized_avg",
            "value": 88.4277305643575,
            "unit": "%",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - go-collector CPU"
          },
          {
            "name": "clickhouse_ram_mib_max",
            "value": 1233.09765625,
            "unit": "MiB",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - clickhouse RAM"
          },
          {
            "name": "go-collector_ram_mib_max",
            "value": 212.94921875,
            "unit": "MiB",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - go-collector RAM"
          },
          {
            "name": "df-engine_cpu_percentage_normalized_avg",
            "value": 45.12946615833279,
            "unit": "%",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - df-engine CPU"
          },
          {
            "name": "clickhouse_cpu_percentage_normalized_avg",
            "value": 49.76302956199225,
            "unit": "%",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - clickhouse CPU"
          },
          {
            "name": "clickhouse_ram_mib_max",
            "value": 1159.4765625,
            "unit": "MiB",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - clickhouse RAM"
          },
          {
            "name": "df-engine_ram_mib_max",
            "value": 52.05078125,
            "unit": "MiB",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - df-engine RAM"
          },
          {
            "name": "test_duration",
            "value": 60.004152,
            "unit": "seconds",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - Test Duration"
          },
          {
            "name": "df-engine_cpu_percentage_normalized_avg",
            "value": 25.544154660099082,
            "unit": "%",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - df-engine CPU"
          },
          {
            "name": "clickhouse_cpu_percentage_normalized_avg",
            "value": 51.42908287070209,
            "unit": "%",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - clickhouse CPU"
          },
          {
            "name": "clickhouse_ram_mib_max",
            "value": 1261.88671875,
            "unit": "MiB",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - clickhouse RAM"
          },
          {
            "name": "df-engine_ram_mib_max",
            "value": 49.9921875,
            "unit": "MiB",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - df-engine RAM"
          },
          {
            "name": "test_duration",
            "value": 60.001956,
            "unit": "seconds",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - Test Duration"
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
        "date": 1787505894058,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "clickhouse_cpu_percentage_normalized_avg",
            "value": 49.16901607711193,
            "unit": "%",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - clickhouse CPU"
          },
          {
            "name": "go-collector_cpu_percentage_normalized_avg",
            "value": 87.63394975491491,
            "unit": "%",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - go-collector CPU"
          },
          {
            "name": "clickhouse_ram_mib_max",
            "value": 1179.30078125,
            "unit": "MiB",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - clickhouse RAM"
          },
          {
            "name": "go-collector_ram_mib_max",
            "value": 182.5,
            "unit": "MiB",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - go-collector RAM"
          },
          {
            "name": "clickhouse_cpu_percentage_normalized_avg",
            "value": 49.85637779334062,
            "unit": "%",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - clickhouse CPU"
          },
          {
            "name": "df-engine_cpu_percentage_normalized_avg",
            "value": 44.693263455385626,
            "unit": "%",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - df-engine CPU"
          },
          {
            "name": "clickhouse_ram_mib_max",
            "value": 1132.40234375,
            "unit": "MiB",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - clickhouse RAM"
          },
          {
            "name": "df-engine_ram_mib_max",
            "value": 54.73046875,
            "unit": "MiB",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - df-engine RAM"
          },
          {
            "name": "test_duration",
            "value": 60.002916,
            "unit": "seconds",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - Test Duration"
          },
          {
            "name": "df-engine_cpu_percentage_normalized_avg",
            "value": 25.011967131171726,
            "unit": "%",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - df-engine CPU"
          },
          {
            "name": "clickhouse_cpu_percentage_normalized_avg",
            "value": 49.985433934461845,
            "unit": "%",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - clickhouse CPU"
          },
          {
            "name": "clickhouse_ram_mib_max",
            "value": 1194.52734375,
            "unit": "MiB",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - clickhouse RAM"
          },
          {
            "name": "df-engine_ram_mib_max",
            "value": 51.8359375,
            "unit": "MiB",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - df-engine RAM"
          },
          {
            "name": "test_duration",
            "value": 60.006384,
            "unit": "seconds",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - Test Duration"
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
        "date": 1787535296203,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "clickhouse_cpu_percentage_normalized_avg",
            "value": 48.78479651738994,
            "unit": "%",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - clickhouse CPU"
          },
          {
            "name": "go-collector_cpu_percentage_normalized_avg",
            "value": 89.48433864939615,
            "unit": "%",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - go-collector CPU"
          },
          {
            "name": "go-collector_ram_mib_max",
            "value": 184.77734375,
            "unit": "MiB",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - go-collector RAM"
          },
          {
            "name": "clickhouse_ram_mib_max",
            "value": 1228.7890625,
            "unit": "MiB",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - clickhouse RAM"
          },
          {
            "name": "df-engine_cpu_percentage_normalized_avg",
            "value": 25.626248697973065,
            "unit": "%",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - df-engine CPU"
          },
          {
            "name": "clickhouse_cpu_percentage_normalized_avg",
            "value": 51.2932994417401,
            "unit": "%",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - clickhouse CPU"
          },
          {
            "name": "clickhouse_ram_mib_max",
            "value": 1219.390625,
            "unit": "MiB",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - clickhouse RAM"
          },
          {
            "name": "df-engine_ram_mib_max",
            "value": 51.0234375,
            "unit": "MiB",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - df-engine RAM"
          },
          {
            "name": "test_duration",
            "value": 60.001854,
            "unit": "seconds",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - Test Duration"
          },
          {
            "name": "df-engine_cpu_percentage_normalized_avg",
            "value": 45.456238574164836,
            "unit": "%",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - df-engine CPU"
          },
          {
            "name": "clickhouse_cpu_percentage_normalized_avg",
            "value": 47.82547879339776,
            "unit": "%",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - clickhouse CPU"
          },
          {
            "name": "clickhouse_ram_mib_max",
            "value": 1179.171875,
            "unit": "MiB",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - clickhouse RAM"
          },
          {
            "name": "df-engine_ram_mib_max",
            "value": 53.40625,
            "unit": "MiB",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - df-engine RAM"
          },
          {
            "name": "test_duration",
            "value": 60.00329,
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
          "id": "3e85c3460361446ebfce99e9f35fffd2dd5ab740",
          "message": "chore(deps): update geneva-uploader digest to 43bd04b (#3867)\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n|---|---|---|---|\n| geneva-uploader | workspace.dependencies | digest | `b4cbfda` →\n`43bd04b` |\n\n---\n\n### Configuration\n\n📅 **Schedule**: (UTC)\n\n- Branch creation\n  - \"before 8am on Monday\"\n- Automerge\n  - At any time (no schedule defined)\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0NC4zOS4wIiwidXBkYXRlZEluVmVyIjoiNDQuMzkuMCIsInRhcmdldEJyYW5jaCI6Im1haW4iLCJsYWJlbHMiOlsiZGVwZW5kZW5jaWVzIl19-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2026-08-24T15:08:06Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/3e85c3460361446ebfce99e9f35fffd2dd5ab740"
        },
        "date": 1787592666200,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "clickhouse_cpu_percentage_normalized_avg",
            "value": 50.782840943075534,
            "unit": "%",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - clickhouse CPU"
          },
          {
            "name": "go-collector_cpu_percentage_normalized_avg",
            "value": 88.20566658465405,
            "unit": "%",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - go-collector CPU"
          },
          {
            "name": "clickhouse_ram_mib_max",
            "value": 1262.58203125,
            "unit": "MiB",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - clickhouse RAM"
          },
          {
            "name": "go-collector_ram_mib_max",
            "value": 186.6640625,
            "unit": "MiB",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - go-collector RAM"
          },
          {
            "name": "df-engine_cpu_percentage_normalized_avg",
            "value": 45.612626156519134,
            "unit": "%",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - df-engine CPU"
          },
          {
            "name": "clickhouse_cpu_percentage_normalized_avg",
            "value": 49.87758733494365,
            "unit": "%",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - clickhouse CPU"
          },
          {
            "name": "clickhouse_ram_mib_max",
            "value": 1231.5390625,
            "unit": "MiB",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - clickhouse RAM"
          },
          {
            "name": "df-engine_ram_mib_max",
            "value": 54.18359375,
            "unit": "MiB",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - df-engine RAM"
          },
          {
            "name": "test_duration",
            "value": 60.003586,
            "unit": "seconds",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - Test Duration"
          },
          {
            "name": "df-engine_cpu_percentage_normalized_avg",
            "value": 25.317298240561954,
            "unit": "%",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - df-engine CPU"
          },
          {
            "name": "clickhouse_cpu_percentage_normalized_avg",
            "value": 52.0729971503978,
            "unit": "%",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - clickhouse CPU"
          },
          {
            "name": "clickhouse_ram_mib_max",
            "value": 1235.69921875,
            "unit": "MiB",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - clickhouse RAM"
          },
          {
            "name": "df-engine_ram_mib_max",
            "value": 53.625,
            "unit": "MiB",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - df-engine RAM"
          },
          {
            "name": "test_duration",
            "value": 60.002403,
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
          "id": "ebc01cc6fb0c71b9830895fbdaffca89607699a5",
          "message": "feat(engine): Add opt-in `size` metric to node produced/consumed (#3842)\n\n# Change summary\n\nClosing the loop on final piece to match the [Go Collector universal\ntelemetry RFC on auto-instrumented\nmetrics](https://github.com/open-telemetry/opentelemetry-collector/blob/main/docs/rfcs/component-universal-telemetry.md#auto-instrumented-metrics)\n🥳\n\nAdds logical payload size to the engine-owned node outcome metrics:\n\n- `node.producer.produced.size{signal,outcome}`\n- `node.consumer.consumed.size{signal,outcome}`\n\nSize follows the existing item-count policy model. `runtime_metrics:\ndetailed` enables both measurements for every node. At `runtime_metrics:\nnormal`, nodes can independently opt in with:\n\n```yaml\npolicies:\n  telemetry:\n    size: true\n```\n\nThe forward path measures the payload once at each produced or consumed\nboundary and stores the value on the context frame. Ack/Nack unwinding\nrecords the stored size with the same signal and outcome as the\ncorresponding message metric.\n\nOTLP payloads report their encoded protobuf length. OTAP payloads report\nlogical Arrow bytes. Cached OTAP sizing avoids repeating the Arrow array\nand buffer walk when the payload is unchanged.\n\nThe `trafficgen-universal-produced-consumed-metrics.yaml` demo includes\nboth item and size policies and prints only produced/consumed node\nmetrics through its internal observability pipeline.\n\n### Sample config run\n\nThe `full` pipeline uses `runtime_metrics: detailed`. Its log sampler\nkeeps one third of log records while metrics and traces pass through\nunchanged:\n\n| Signal | Receiver produced | Sampler consumed | Sampler produced |\nNoop consumed |\n| --- | ---: | ---: | ---: | ---: |\n| Logs messages | 4 | 4 | 4 | 4 |\n| Logs items | 30 | 30 | 10 | 10 |\n| Logs size (By) | 7,472 | 7,472 | 6,448 | 6,448 |\n| Metrics messages | 2 | 2 | 2 | 2 |\n| Metrics items | 18 | 18 | 18 | 18 |\n| Metrics size (By) | 3,710 | 3,710 | 3,710 | 3,710 |\n| Traces messages | 2 | 2 | 2 | 2 |\n| Traces items | 12 | 12 | 12 | 12 |\n| Traces size (By) | 2,290 | 2,290 | 2,290 | 2,290 |\n\nDropping two thirds of the log records reduces logical size from `7,472\nBy` to `6,448 By` rather than by two thirds. OTAP is a columnar,\nrelational representation: resource, scope, attribute, and dictionary\nbuffers remain largely unchanged, while sampling removes primarily the\nper-record offsets and dictionary keys. The synthetic records also\nrepeat values that are stored once in dictionaries. Logical Arrow size\ntherefore describes the resulting representation, not an average record\nsize multiplied by the item count.\n\nThe `partial` pipeline uses `runtime_metrics: normal` and opts only the\nsampler into `item_counts` and `size`:\n\n| Node boundary | Messages (logs / metrics / traces) | Items (logs /\nmetrics / traces) | Size in By (logs / metrics / traces) |\n| --- | ---: | ---: | ---: |\n| Receiver produced | 4 / 2 / 2 | Not present | Not present |\n| Sampler consumed | 4 / 2 / 2 | 30 / 18 / 12 | 7,472 / 3,710 / 2,290 |\n| Sampler produced | 4 / 2 / 2 | 10 / 18 / 12 | 6,448 / 3,710 / 2,290 |\n| Noop consumed | 4 / 2 / 2 | Not present | Not present |\n\n## Related issue\n\n* Closes #2884\n\n## Validation\n\nLocal engine runs\n\n## User-facing changes\n\nYes. Users can enable node-level logical payload size metrics globally\nwith `runtime_metrics: detailed` or per node with\n`policies.telemetry.size: true`.",
          "timestamp": "2026-08-24T22:34:55Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/ebc01cc6fb0c71b9830895fbdaffca89607699a5"
        },
        "date": 1787623791839,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "clickhouse_cpu_percentage_normalized_avg",
            "value": 49.34063489790025,
            "unit": "%",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - clickhouse CPU"
          },
          {
            "name": "go-collector_cpu_percentage_normalized_avg",
            "value": 89.93502713292112,
            "unit": "%",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - go-collector CPU"
          },
          {
            "name": "go-collector_ram_mib_max",
            "value": 192.9140625,
            "unit": "MiB",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - go-collector RAM"
          },
          {
            "name": "clickhouse_ram_mib_max",
            "value": 1245.66015625,
            "unit": "MiB",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - clickhouse RAM"
          },
          {
            "name": "clickhouse_cpu_percentage_normalized_avg",
            "value": 48.79832436976959,
            "unit": "%",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - clickhouse CPU"
          },
          {
            "name": "df-engine_cpu_percentage_normalized_avg",
            "value": 44.77143865750355,
            "unit": "%",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - df-engine CPU"
          },
          {
            "name": "clickhouse_ram_mib_max",
            "value": 1200.10546875,
            "unit": "MiB",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - clickhouse RAM"
          },
          {
            "name": "df-engine_ram_mib_max",
            "value": 47.8125,
            "unit": "MiB",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - df-engine RAM"
          },
          {
            "name": "test_duration",
            "value": 60.003156,
            "unit": "seconds",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - Test Duration"
          },
          {
            "name": "df-engine_cpu_percentage_normalized_avg",
            "value": 25.56258675748441,
            "unit": "%",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - df-engine CPU"
          },
          {
            "name": "clickhouse_cpu_percentage_normalized_avg",
            "value": 50.37044936520459,
            "unit": "%",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - clickhouse CPU"
          },
          {
            "name": "clickhouse_ram_mib_max",
            "value": 1177.62890625,
            "unit": "MiB",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - clickhouse RAM"
          },
          {
            "name": "df-engine_ram_mib_max",
            "value": 54.4375,
            "unit": "MiB",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - df-engine RAM"
          },
          {
            "name": "test_duration",
            "value": 60.007156,
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
          "id": "ebc01cc6fb0c71b9830895fbdaffca89607699a5",
          "message": "feat(engine): Add opt-in `size` metric to node produced/consumed (#3842)\n\n# Change summary\n\nClosing the loop on final piece to match the [Go Collector universal\ntelemetry RFC on auto-instrumented\nmetrics](https://github.com/open-telemetry/opentelemetry-collector/blob/main/docs/rfcs/component-universal-telemetry.md#auto-instrumented-metrics)\n🥳\n\nAdds logical payload size to the engine-owned node outcome metrics:\n\n- `node.producer.produced.size{signal,outcome}`\n- `node.consumer.consumed.size{signal,outcome}`\n\nSize follows the existing item-count policy model. `runtime_metrics:\ndetailed` enables both measurements for every node. At `runtime_metrics:\nnormal`, nodes can independently opt in with:\n\n```yaml\npolicies:\n  telemetry:\n    size: true\n```\n\nThe forward path measures the payload once at each produced or consumed\nboundary and stores the value on the context frame. Ack/Nack unwinding\nrecords the stored size with the same signal and outcome as the\ncorresponding message metric.\n\nOTLP payloads report their encoded protobuf length. OTAP payloads report\nlogical Arrow bytes. Cached OTAP sizing avoids repeating the Arrow array\nand buffer walk when the payload is unchanged.\n\nThe `trafficgen-universal-produced-consumed-metrics.yaml` demo includes\nboth item and size policies and prints only produced/consumed node\nmetrics through its internal observability pipeline.\n\n### Sample config run\n\nThe `full` pipeline uses `runtime_metrics: detailed`. Its log sampler\nkeeps one third of log records while metrics and traces pass through\nunchanged:\n\n| Signal | Receiver produced | Sampler consumed | Sampler produced |\nNoop consumed |\n| --- | ---: | ---: | ---: | ---: |\n| Logs messages | 4 | 4 | 4 | 4 |\n| Logs items | 30 | 30 | 10 | 10 |\n| Logs size (By) | 7,472 | 7,472 | 6,448 | 6,448 |\n| Metrics messages | 2 | 2 | 2 | 2 |\n| Metrics items | 18 | 18 | 18 | 18 |\n| Metrics size (By) | 3,710 | 3,710 | 3,710 | 3,710 |\n| Traces messages | 2 | 2 | 2 | 2 |\n| Traces items | 12 | 12 | 12 | 12 |\n| Traces size (By) | 2,290 | 2,290 | 2,290 | 2,290 |\n\nDropping two thirds of the log records reduces logical size from `7,472\nBy` to `6,448 By` rather than by two thirds. OTAP is a columnar,\nrelational representation: resource, scope, attribute, and dictionary\nbuffers remain largely unchanged, while sampling removes primarily the\nper-record offsets and dictionary keys. The synthetic records also\nrepeat values that are stored once in dictionaries. Logical Arrow size\ntherefore describes the resulting representation, not an average record\nsize multiplied by the item count.\n\nThe `partial` pipeline uses `runtime_metrics: normal` and opts only the\nsampler into `item_counts` and `size`:\n\n| Node boundary | Messages (logs / metrics / traces) | Items (logs /\nmetrics / traces) | Size in By (logs / metrics / traces) |\n| --- | ---: | ---: | ---: |\n| Receiver produced | 4 / 2 / 2 | Not present | Not present |\n| Sampler consumed | 4 / 2 / 2 | 30 / 18 / 12 | 7,472 / 3,710 / 2,290 |\n| Sampler produced | 4 / 2 / 2 | 10 / 18 / 12 | 6,448 / 3,710 / 2,290 |\n| Noop consumed | 4 / 2 / 2 | Not present | Not present |\n\n## Related issue\n\n* Closes #2884\n\n## Validation\n\nLocal engine runs\n\n## User-facing changes\n\nYes. Users can enable node-level logical payload size metrics globally\nwith `runtime_metrics: detailed` or per node with\n`policies.telemetry.size: true`.",
          "timestamp": "2026-08-24T22:34:55Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/ebc01cc6fb0c71b9830895fbdaffca89607699a5"
        },
        "date": 1787679275297,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "clickhouse_cpu_percentage_normalized_avg",
            "value": 47.06185500004253,
            "unit": "%",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - clickhouse CPU"
          },
          {
            "name": "go-collector_cpu_percentage_normalized_avg",
            "value": 87.8054039962526,
            "unit": "%",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - go-collector CPU"
          },
          {
            "name": "clickhouse_ram_mib_max",
            "value": 1224.60546875,
            "unit": "MiB",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - clickhouse RAM"
          },
          {
            "name": "go-collector_ram_mib_max",
            "value": 188.296875,
            "unit": "MiB",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - go-collector RAM"
          },
          {
            "name": "df-engine_cpu_percentage_normalized_avg",
            "value": 45.517370637648355,
            "unit": "%",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - df-engine CPU"
          },
          {
            "name": "clickhouse_cpu_percentage_normalized_avg",
            "value": 49.96152953413885,
            "unit": "%",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - clickhouse CPU"
          },
          {
            "name": "clickhouse_ram_mib_max",
            "value": 1163.89453125,
            "unit": "MiB",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - clickhouse RAM"
          },
          {
            "name": "df-engine_ram_mib_max",
            "value": 56.23828125,
            "unit": "MiB",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - df-engine RAM"
          },
          {
            "name": "test_duration",
            "value": 60.00502,
            "unit": "seconds",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - Test Duration"
          },
          {
            "name": "df-engine_cpu_percentage_normalized_avg",
            "value": 24.9734622667159,
            "unit": "%",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - df-engine CPU"
          },
          {
            "name": "clickhouse_cpu_percentage_normalized_avg",
            "value": 48.60340277274036,
            "unit": "%",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - clickhouse CPU"
          },
          {
            "name": "clickhouse_ram_mib_max",
            "value": 1124.19140625,
            "unit": "MiB",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - clickhouse RAM"
          },
          {
            "name": "df-engine_ram_mib_max",
            "value": 51.4375,
            "unit": "MiB",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - df-engine RAM"
          },
          {
            "name": "test_duration",
            "value": 60.00199,
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
          "id": "a942658eab44e66650313ecdce8ca90a924feae7",
          "message": "fix(metrics): Simplify engine-owned node and flow metric names (#3853)\n\n# Change summary\n\nSimplify engine-owned node and flow metric names around the direction of\ndata through the pipeline:\n\n- `node.consumer.consumed.*` becomes `node.input.*`\n- `node.producer.produced.*` becomes `node.output.*`\n- `flow.consumed.*` becomes `flow.input.*`\n- `flow.produced.*` becomes `flow.output.*`\n\nAlso:\n\n- Add `messages` and logical payload `size` measurements to flow input\nand output metrics alongside `items`.\n- Report `flow.compute.duration` as a seconds-based exponential\nhistogram while retaining its processor-compute semantics.\n- Gate flow measurements with a compact interest bitmap so disabled item\nand size metrics do not inspect PData.\n- Consolidate the node-versus-flow example as\n`trafficgen-input-output-metrics.yaml`.\n- Align Rust flow metric types, fields, methods, tests, documentation,\nand configuration values with input/output terminology.\n\n## Related issue\n\n- Related to #3300\n\n### Validation\n\nThe `trafficgen-input-output-metrics.yaml` example produced the\nfollowing representative one-second delta interval. All node rows have\n`outcome=success`.\n\nThe `full` pipeline enables detailed node metrics and a one-processor\nflow around `sampler`:\n\n```text\nreceiver output\n      │\n      ├── sampler node input == flow input\n      │            30 logs in\n      │            20 logs dropped\n      │            10 logs out\n      └── sampler node output == flow output == noop node input\n```\n\nNode and flow metric sets are stacked in pipeline order. Every value\ncell is `traces / metrics / logs`.\n\n| Pipeline boundary | Metric scope | `messages` | `items` | `size` (By)\n|\n| --- | --- | ---: | ---: | ---: |\n| Receiver output | `node.output` | `1 / 2 / 4` | `6 / 18 / 30` | `1145\n/ 3710 / 7472` |\n| Sampler input | `node.input` | `1 / 2 / 4` | `6 / 18 / 30` | `1145 /\n3710 / 7472` |\n| Flow input | `flow.input` | `1 / 2 / 4` | `6 / 18 / 30` | `1145 / 3710\n/ 7472` |\n| Flow output | `flow.output` | `1 / 2 / 4` | `6 / 18 / 10` | `1145 /\n3710 / 6448` |\n| Sampler output | `node.output` | `1 / 2 / 4` | `6 / 18 / 10` | `1145 /\n3710 / 6448` |\n| Flow decision | `flow.dropped` | — | `— / — / 20` | — |\n| Noop input | `node.input` | `1 / 2 / 4` | `6 / 18 / 10` | `1145 / 3710\n/ 6448` |\n\nThe stacked rows show both boundary agreements and the sampler\ntransformation: `30 log items input - 20 dropped = 10 output`.\n\nDuration histograms use seconds. Each cell is `count / sum / min / max`.\n\n| Measurement | Traces | Metrics | Logs |\n| --- | ---: | ---: | ---: |\n| Receiver `node.output.duration` | `1 / 0.0002026 / 0.0002026 /\n0.0002026` | `2 / 0.0003946 / 0.0001809 / 0.0002137` | `4 / 0.0064401 /\n0.0009315 / 0.0022869` |\n| Sampler `node.input.duration` | `1 / 0.0001319 / 0.0001319 /\n0.0001319` | `2 / 0.0002355 / 0.0000921 / 0.0001434` | `4 / 0.005842 /\n0.0008909 / 0.0020428` |\n| Sampler `flow.compute.duration` | `1 / 0.000009 / 0.000009 / 0.000009`\n| `2 / 0.0000223 / 0.0000109 / 0.0000114` | `4 / 0.0053347 / 0.0008101 /\n0.0018651` |\n| Noop `node.input.duration` | `1 / 0.0000063 / 0.0000063 / 0.0000063` |\n`2 / 0.0000189 / 0.000009 / 0.0000099` | `4 / 0.0000316 / 0.0000062 /\n0.000009` |\n\nOver the course of implementation, I was confused about what\n`node.*.duration` actually represented, opened follow-up issue to\nimprove:\n- https://github.com/open-telemetry/otel-arrow/issues/3881\n\nThe `partial` pipeline uses normal runtime metrics and opts only the\nsampler into item and size measurements. Each signal cell remains\n`messages / items / size in bytes`; `—` means that instrument is\ndisabled.\n\n| Boundary | Scope | Traces | Metrics | Logs |\n| --- | --- | ---: | ---: | ---: |\n| Receiver output | `node.output` | `1 / — / —` | `2 / — / —` | `4 / — /\n—` |\n| Sampler input | `node.input` | `1 / 6 / 1145` | `2 / 18 / 3710` | `4 /\n30 / 7472` |\n| Sampler output | `node.output` | `1 / 6 / 1145` | `2 / 18 / 3710` | `4\n/ 10 / 6448` |\n| Noop input | `node.input` | `1 / — / —` | `2 / — / —` | `4 / — / —` |\n\nThe partial pipeline emits no flow scopes or node duration histograms.\n\nThe `no_output` pipeline uses a deterministic filter that drops every\ngenerated log and ACKs without sending. A representative snapshot\ncontained 4 input messages with 40 items and 9792 logical bytes:\n\n| Pipeline boundary | Metric scope | `messages` | `items` | `size` (By)\n|\n| --- | --- | ---: | ---: | ---: |\n| Receiver output | `node.output` | `4` | `40` | `9792` |\n| Filter input | `node.input` | `4` | `40` | `9792` |\n| Flow input | `flow.input` | `4` | `40` | `9792` |\n| Flow decision | `flow.dropped` | — | `40` | — |\n| Flow output | `flow.output` | absent | absent | absent |\n| Filter output | `node.output` | absent | absent | absent |\n| Noop input | `node.input` | absent | absent | absent |\n\n`flow.compute.duration` still records the completed processor work:\n`count=4`, `sum=0.006460706 s`, `min=0.001050001 s`, and\n`max=0.002712202 s`. This demonstrates that an ACK without a send\nfinalizes flow compute and drop accounting without inventing an output\nmessage.\n\n## User-facing changes\n\nReplace:\n\n- `node.consumer.consumed.*` with `node.input.*`\n- `node.producer.produced.*` with `node.output.*`\n- `flow.consumed.*` with `flow.input.*`\n- `flow.produced.*` with `flow.output.*`\n- Flow metric configuration values based on consumed/produced\nterminology with `input_messages`, `input_items`, `input_size`,\n`output_messages`, `output_items`, and `output_size`.\n- Scope selectors targeting the previous names with the corresponding\ninput/output scopes.\n\nTreat `flow.compute.duration` values as seconds and as histogram\nobservations.",
          "timestamp": "2026-08-25T22:23:10Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/a942658eab44e66650313ecdce8ca90a924feae7"
        },
        "date": 1787708122908,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "clickhouse_cpu_percentage_normalized_avg",
            "value": 47.98397953063843,
            "unit": "%",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - clickhouse CPU"
          },
          {
            "name": "go-collector_cpu_percentage_normalized_avg",
            "value": 89.27734023072973,
            "unit": "%",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - go-collector CPU"
          },
          {
            "name": "clickhouse_ram_mib_max",
            "value": 1207.39453125,
            "unit": "MiB",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - clickhouse RAM"
          },
          {
            "name": "go-collector_ram_mib_max",
            "value": 199.3359375,
            "unit": "MiB",
            "extra": "OTel Collector ClickHouse Logs/OTELCOL-OTLP-TRANSFORMED-100K - go-collector RAM"
          },
          {
            "name": "clickhouse_cpu_percentage_normalized_avg",
            "value": 50.754899447741394,
            "unit": "%",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - clickhouse CPU"
          },
          {
            "name": "df-engine_cpu_percentage_normalized_avg",
            "value": 25.576977500238236,
            "unit": "%",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - df-engine CPU"
          },
          {
            "name": "clickhouse_ram_mib_max",
            "value": 1251.67578125,
            "unit": "MiB",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - clickhouse RAM"
          },
          {
            "name": "df-engine_ram_mib_max",
            "value": 56.578125,
            "unit": "MiB",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - df-engine RAM"
          },
          {
            "name": "test_duration",
            "value": 60.001827,
            "unit": "seconds",
            "extra": "ClickHouse OTAP Logs/OTAP-IN-BATCHED-100K - Test Duration"
          },
          {
            "name": "df-engine_cpu_percentage_normalized_avg",
            "value": 45.752438386686364,
            "unit": "%",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - df-engine CPU"
          },
          {
            "name": "clickhouse_cpu_percentage_normalized_avg",
            "value": 48.18815774970495,
            "unit": "%",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - clickhouse CPU"
          },
          {
            "name": "clickhouse_ram_mib_max",
            "value": 1155,
            "unit": "MiB",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - clickhouse RAM"
          },
          {
            "name": "df-engine_ram_mib_max",
            "value": 52.7421875,
            "unit": "MiB",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - df-engine RAM"
          },
          {
            "name": "test_duration",
            "value": 60.004164,
            "unit": "seconds",
            "extra": "ClickHouse OTAP Logs/OTLP-IN-BATCHED-100K - Test Duration"
          }
        ]
      }
    ]
  }
}