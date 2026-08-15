window.BENCHMARK_DATA = {
  "lastUpdate": 1786816077178,
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
      }
    ]
  }
}