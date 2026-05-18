window.BENCHMARK_DATA = {
  "lastUpdate": 1779118187875,
  "repoUrl": "https://github.com/open-telemetry/otel-arrow",
  "entries": {
    "Benchmark": [
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
          "distinct": false,
          "id": "0884c8c9d267ef131a68e208191adba0c4679ce9",
          "message": "Validation framework fix equivalence check for metrics (#2776)\n\n# Change Summary\n\nThis adds additional validation tests to check that all signals are\npreserved through otlp/otap receiver/exporters.\n\nAdds canonicalize_buckets fn to set values from None to Some(Buckets\n{offset: 0, bucket_counts: vec![]}). Before the validation tests\ninvolving metrics and otap receiver/exporters would fail, but now they\nare succeeding\n\n## What issue does this PR close?\n\n* Closes #2786\n\n## How are these changes tested?\n\nvalidation test\n\n## Are there any user-facing changes?\n\nno",
          "timestamp": "2026-04-30T18:31:18Z",
          "tree_id": "1738573548c52c15c3eb31768b9559338bd5b52c",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/0884c8c9d267ef131a68e208191adba0c4679ce9"
        },
        "date": 1777586944071,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.703234851360321,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 5.73870144245342,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 6.233022037293882,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.167708333333334,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.4375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 6066.5595735276875,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 6109.221736492017,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.006334,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 214024.00320831416,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 176900.03338986047,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
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
          "distinct": false,
          "id": "c8804f567d53c195a25f3be40080e281edcb2314",
          "message": "docs: consolidate CLAUDE.md and copilot-instructions.md into AGENTS.md (#2791)\n\nBoth `CLAUDE.md` and `.github/copilot-instructions.md` were stubs that\nsimply redirected to\n[rust/otap-dataflow/AGENTS.md](rust/otap-dataflow/AGENTS.md).\n\nModern AI coding agents (Claude Code, GitHub Copilot, Cursor, etc.) all\nsupport a single top-level `AGENTS.md` file, so this consolidates the\ntwo stubs into one.",
          "timestamp": "2026-04-30T18:59:36Z",
          "tree_id": "4647005286ec671e7c219b6c24b342409c74ba3e",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/c8804f567d53c195a25f3be40080e281edcb2314"
        },
        "date": 1777589045242,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.9887005090713501,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 5.740904093832289,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 6.202992944095526,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.544921875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.75,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 6041.3450552386685,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5981.614242545631,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002532,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 212410.94907589044,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 175491.70640271768,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "107717825+opentelemetrybot@users.noreply.github.com",
            "name": "OpenTelemetry Bot",
            "username": "opentelemetrybot"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "7af33d651e8e9db5b2a7775d0cb106c1e12027c1",
          "message": "chore: Move inactive members to emeritus (#2789)\n\n## Move inactive members to emeritus\n\nThe following members have had no activity in\n`open-telemetry/otel-arrow` since **2025-12-31** and are being moved to\nemeritus:\n\n- @v0y4g3r (Approver, Remove from team(s): arrow-approvers,\narrow-triagers)\n\n> [!IMPORTANT]\n> After merging, remove the user(s) from:\n> - The listed team(s) in GitHub\n> - Any relevant private channels on Slack\n> - Any relevant package managers used for publishing\n\nThis PR was automatically generated by the [move-to-emeritus\nworkflow](https://github.com/open-telemetry/community/actions/workflows/move-to-emeritus.yml).",
          "timestamp": "2026-04-30T19:03:28Z",
          "tree_id": "1b55f64569a732ae3704f30b49e3638d170946dc",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/7af33d651e8e9db5b2a7775d0cb106c1e12027c1"
        },
        "date": 1777591981281,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.8462623953819275,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 5.755992704319522,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 6.221231816774992,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.548697916666665,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.7890625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 6049.779522060951,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 6100.976527889394,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003509,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 213217.70165803612,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 176318.92045398292,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "43051891+hestolz@users.noreply.github.com",
            "name": "Henry Stolz",
            "username": "hestolz"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "74bcd9886b81855f59df803b3e8979f35115cceb",
          "message": "fix crate name in otap-dataflow readme (#2790)\n\n# Change Summary\n\nfix crate name in otap-dataflow readme.\n\n## What issue does this PR close?\n\nminor nit.\n\n## How are these changes tested?\n\nN/A\n\n## Are there any user-facing changes?\n\nN/A",
          "timestamp": "2026-04-30T19:39:37Z",
          "tree_id": "6ed9c7961fd84cc2a5d0ccb4882e64ef1be54275",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/74bcd9886b81855f59df803b3e8979f35115cceb"
        },
        "date": 1777599774122,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.1299434900283813,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 5.786249524290974,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 6.187954791763431,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.77109375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.7578125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 6041.257057974342,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 6109.519849589872,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003406,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 214190.9700314921,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 177284.5967679228,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
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
          "id": "993fa369f3ef1c12b381f01f758302e8243f211a",
          "message": "feat(engine): Extension System - Capability Registry & Resolver (#2732)\n\n# Extension system — Phase 1 (capabilities, registry, builder, proc\nmacro)\n\nImplements the Phase-1 extension/capability system for the OTAP dataflow\nengine. Extensions are first-class config siblings of nodes; nodes\nexplicitly bind to extension instances via named capabilities, and\nreceive typed handles resolved once at factory time — no hot-path\nregistry lookups.\n\nTracking docs:\n\n[`docs/extension-system-architecture.md`](rust/otap-dataflow/docs/extension-system-architecture.md)\n(rewritten in this PR).\n\n## What's in this PR\n\n### `#[capability]` proc macro (`engine-macros`)\n\n- New `capability.rs` expansion: from a single `#[capability] trait\n  Foo { ... }` source it generates `local::Foo` (`!Send`-friendly) and\n  `shared::Foo` (`Send + Clone`) trait variants plus a `SharedAsLocal`\n  adapter and an `ExtensionCapability` impl. The dual variants are\n  derived from one source, so authors can't accidentally let local\n  and shared semantics diverge.\n- New `pipeline_factory.rs` expansion to build the static\n  `PipelineFactory` registry used by `main.rs`.\n- All emitted paths use fully-qualified `::std::...` /\n  `::async_trait::...` / `::otap_df_engine::...` so generated code is\n  hygienic in any caller crate.\n\n### Capability registry (`engine::capability`)\n\n- `CapabilityRegistry`: typed-keyed (`(extension_name, TypeId)`)\n  storage with **typestate-enforced** single `.shared()` / `.local()`\n  registration per builder — duplicates are unrepresentable rather\n  than runtime errors.\n- Two execution models: native local (`Rc<dyn Local>`, lock-free) and\n  native shared (`Box<dyn Shared>`, `Send + Clone`). A shared-only\n  extension serves local consumers transparently via the\n  `SharedAsLocal` adapter generated by the proc macro.\n- Two **instance policies** chosen at build time, invisible to\n  consumers: `.cloned()` (clone a stored prototype) and\n  `.constructed()` (per-consumer construction via a closure;\n  Passive-only — `Active + Constructed` is statically rejected).\n- `resolve_bindings`: walks a node's `capabilities:` declaration and\n  produces a per-node `Capabilities` bundle with all bindings\n  resolved, surfacing config errors (unknown extension, unknown\n  capability, capability not provided by bound extension, multiple\n  bindings for the same capability).\n- `Capabilities`: per-node consumer API with `require_local`,\n  `require_shared`, `optional_local`, `optional_shared`. Instances\n  are minted lazily at the call site, not at resolution time.\n- `ConsumedTracker`: cross-node, per-(capability, extension)\n  consumption flags driving `drop_local()` / `drop_shared()` cleanup\n  for extensions no node ever claimed.\n\n### One-shot consumption contract\n\nA binding is claimable **at most once per node**, regardless of\nexecution model. The guard is the `Cell<Option<_>>::take()` on each\nresolved entry's `produce` closure — no auxiliary flag.\n\n- Same accessor twice → `CapabilityAlreadyConsumed`.\n- Different accessors on a SharedAsLocal-fallback binding share one\n  underlying entry, so claiming either side consumes the other\n  naturally.\n- Different accessors on a native-dual binding (extension registered\n  both native local **and** native shared) take and drop the\n  alternative entry's `produce` closure on success, so the\n  per-binding contract holds uniformly. The cross-node tracker is\n  only flipped by actual consumption, not by invalidation, so\n  `drop_*` cleanup remains correct.\n\n### Documentation\n\n- `docs/extension-system-architecture.md`: rewritten to describe the\n  capability-based design, the local/shared duality, instance\n  policies, Active vs Passive lifecycle, and the typestate builder.\n\n## Tests\n\nNew, focused unit tests cover:\n\n- Registry: typestate single-registration, duplicate rejection,\n  `SharedAsLocal` adapter freshness per node, double-`Box` envelope\n  for shared `produce`.\n- `resolve_bindings`: every error path (unknown extension / unknown\n  capability / capability not provided / wrong extension), local-only\n  and shared-only binding shapes, fallback path, native-dual path.\n- One-shot contract: second-call rejection on each of `require_local`,\n  `require_shared`, `optional_local`, `optional_shared`; fallback\n  cross-side rejection; native-dual cross-side rejection (both\n  directions).\n- `ConsumedTracker`: per-extension consumption flags, with the\n  invariant that mere invalidation does not flip a bucket.\n- Proc-macro end-to-end: `local-only`, `shared-only`, and `dual`\n  forms of `extension_capabilities!` against the registry.\n\n## Validation\n\n```text\ncargo xtask check\n✅ Cargo workspace structure complies with project policies.\n✅ Formatting completed successfully.\n✅ Clippy linting passed without warnings.\n✅ All tests passed successfully.\n```\n\n---------\n\nCo-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>",
          "timestamp": "2026-05-01T02:10:49Z",
          "tree_id": "29c54668ae7178d41f61630f34daf75d7a38356a",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/993fa369f3ef1c12b381f01f758302e8243f211a"
        },
        "date": 1777613850314,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.2747875452041626,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 5.777533315105428,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 6.252938271604938,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.255859375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.66015625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 6024.176400881582,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 6100.971850751176,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003555,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 214246.1013853716,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 177289.22975114422,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "cithomas@microsoft.com",
            "name": "Cijo Thomas",
            "username": "cijothomas"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "ce3582596886edf41cb83c87829be8cd8d15fcce",
          "message": "fix(perf-test): add missing /api/v1 prefix to idle-state template endpoints (#2798)\n\nThe idle-state-template.yaml.j2 was using /telemetry/metrics instead of\n/api/v1/telemetry/metrics for both the Prometheus scraping endpoint and\nthe ready-check URL. This caused 404 errors during idle state\nbenchmarks.\n\nAll other test configs already had the correct /api/v1 prefix.\n\n# Change Summary\n\n<!--\nReplace with a brief summary of the change in this PR\n-->\n\n## What issue does this PR close?\n\n<!--\nWe highly recommend correlation of every PR to an issue\n-->\n\n* Closes #NNN\n\n## How are these changes tested?\n\n## Are there any user-facing changes?\n\n <!-- If yes, provide further info below -->",
          "timestamp": "2026-05-01T16:38:12Z",
          "tree_id": "a86e34dbb8f8829478ea49a921a5480b991c9701",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/ce3582596886edf41cb83c87829be8cd8d15fcce"
        },
        "date": 1777660662196,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.7062146663665771,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 5.717974139325335,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 6.146508254898935,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.654557291666666,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.84765625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 6041.235512124102,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 6083.899604723849,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00362,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 212600.57802859842,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 175627.8204025501,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "213113461+gyanranjanpanda@users.noreply.github.com",
            "name": "Gyan ranjan",
            "username": "gyanranjanpanda"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "f018901f3a2ba93bee9d45f6afb1204f90679863",
          "message": "Fix duplicate attribute keys in transform_attributes (#2423)\n\n# Fix Duplicate Attribute Keys in `transform_attributes`\n\n## Changes Made\nThis PR resolves issue #1650 by ensuring that dictionary keys are\ndeduplicated when transformations such as `rename` are applied, as\nrequired by the OpenTelemetry specification (\"Exported maps MUST contain\nonly unique keys by default\").\n\nTo accomplish this while maintaining strict performance requirements, we\nreplaced the previous `RowConverter` deduplication strategy with a new\nhigh-performance, proactive pre-filter:\n- We injected `filter_rename_collisions` into\n`transform_attributes_impl` inside\n`otap-dataflow/crates/pdata/src/otap/transform.rs`.\n- Before a rename is processed, this function reads the `parent_id`s and\ntarget keys. It uses the `IdBitmap` type to find any existing target\nkeys whose `parent_id` maps back to an old key that will be renamed.\n- It proactively strips those collision rows from the batch via\n`arrow::compute::filter_record_batch` *before* the actual transform\nhappens.\n\n## Testing\n- Extended the `AttributesProcessor` unit tests\n(`test_rename_removes_duplicate_keys`) to explicitly verify that\nrenaming an attribute resulting in a collision automatically discards\nduplicate keys.\n- Extended the `AttributesTransformPipelineStage` in `query-engine`\ntests with a parallel case ensuring OPL/KQL query pipelines\n(`project-rename`) properly drop duplicates when resolving duplicates.\n- Refactored `otap_df_pdata` `transform.rs` tests to properly expect\ndeduplicated keys using this plan-based method.\n- Validated logic with `cargo test --workspace --all-features`.\n\n## Validation Results\nAll tests pass. OTel semantic rules surrounding unique mapped keys map\ncleanly through down/upstream processors. The `IdBitmap` intersection\napproach completely resolves the multi-thousand percent `RowConverter`\nperformance regressions, dropping collision resolution overhead to\nessentially zero through efficient bitmap operations.\n\n---------\n\nSigned-off-by: Gyanranjan Panda <gyanranjanpanda438@gmail.com>\nCo-authored-by: Gyan Ranjan Panda <gyanranjanpanda@users.noreply.github.com>\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-05-01T20:08:07Z",
          "tree_id": "e2085c4a3fb15d509fdc667a56549abc71c1a226",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/f018901f3a2ba93bee9d45f6afb1204f90679863"
        },
        "date": 1777672940645,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.8486563563346863,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 5.7666135124231,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 6.32263137528055,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.584635416666668,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.59765625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 6032.7244101018,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 6083.92150551992,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003404,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 213617.5025300227,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 176650.40525475284,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
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
          "id": "9aa767ee7b26712bbab69e4ecab5db2b22f80f32",
          "message": "Update github workflow dependencies (#2802)\n\n> ℹ️ **Note**\n> \n> This PR body was truncated due to platform limits.\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n|---|---|---|---|\n|\n[DavidAnson/markdownlint-cli2-action](https://redirect.github.com/DavidAnson/markdownlint-cli2-action)\n| action | minor | `v23.0.0` → `v23.1.0` |\n|\n[EmbarkStudios/cargo-deny-action](https://redirect.github.com/EmbarkStudios/cargo-deny-action)\n| action | patch | `v2.0.15` → `v2.0.17` |\n| [Swatinem/rust-cache](https://redirect.github.com/Swatinem/rust-cache)\n| action | minor | `v2` → `v2.9.1` |\n|\n[actions/create-github-app-token](https://redirect.github.com/actions/create-github-app-token)\n| action | minor | `v3.0.0` → `v3.1.1` |\n| [actions/setup-node](https://redirect.github.com/actions/setup-node) |\naction | minor | `v6.3.0` → `v6.4.0` |\n|\n[actions/upload-artifact](https://redirect.github.com/actions/upload-artifact)\n| action | patch | `v7.0.0` → `v7.0.1` |\n|\n[github/codeql-action](https://redirect.github.com/github/codeql-action)\n| action | patch | `v4.35.1` → `v4.35.3` |\n| [go](https://redirect.github.com/actions/go-versions) | uses-with |\npatch | `1.26.1` → `1.26.2` |\n|\n[step-security/harden-runner](https://redirect.github.com/step-security/harden-runner)\n| action | minor | `v2.16.1` → `v2.19.0` |\n|\n[taiki-e/install-action](https://redirect.github.com/taiki-e/install-action)\n| action | minor | `v2.71.2` → `v2.75.28` |\n\n---\n\n### Release Notes\n\n<details>\n<summary>DavidAnson/markdownlint-cli2-action\n(DavidAnson/markdownlint-cli2-action)</summary>\n\n###\n[`v23.1.0`](https://redirect.github.com/DavidAnson/markdownlint-cli2-action/releases/tag/v23.1.0):\nUpdate markdownlint-cli2 version (markdownlint-cli2 v0.22.1,\nmarkdownlint v0.40.0).\n\n[Compare\nSource](https://redirect.github.com/DavidAnson/markdownlint-cli2-action/compare/v23.0.0...v23.1.0)\n\n</details>\n\n<details>\n<summary>EmbarkStudios/cargo-deny-action\n(EmbarkStudios/cargo-deny-action)</summary>\n\n###\n[`v2.0.17`](https://redirect.github.com/EmbarkStudios/cargo-deny-action/releases/tag/v2.0.17):\nRelease 2.0.17 - cargo-deny 0.19.2\n\n[Compare\nSource](https://redirect.github.com/EmbarkStudios/cargo-deny-action/compare/v2.0.16...v2.0.17)\n\n##### Fixed\n\n-\n[PR#845](https://redirect.github.com/EmbarkStudios/cargo-deny/pull/845)\nfixed structural issues with SARIF output, resolving\n[#&#8203;818](https://redirect.github.com/EmbarkStudios/cargo-deny/issues/818).\nThanks\n[@&#8203;KyleChamberlin](https://redirect.github.com/KyleChamberlin)!\n\n###\n[`v2.0.16`](https://redirect.github.com/EmbarkStudios/cargo-deny-action/releases/tag/v2.0.16):\nRelease 2.0.16 - cargo-deny 0.19.1\n\n[Compare\nSource](https://redirect.github.com/EmbarkStudios/cargo-deny-action/compare/v2.0.15...v2.0.16)\n\n##### Fixed\n\n-\n[PR#833](https://redirect.github.com/EmbarkStudios/cargo-deny/pull/833)\nfixed an issue where the maximum advisory database staleness was over 14\nyears instead of the intended 90 days.\n-\n[PR#839](https://redirect.github.com/EmbarkStudios/cargo-deny/pull/839)\nfixed an issue where unsound advisories would appear for transitive\ndependencies despite requesting them only for workspace dependencies,\nresolving\n[#&#8203;829](https://redirect.github.com/EmbarkStudios/cargo-deny/issues/829).\n-\n[PR#840](https://redirect.github.com/EmbarkStudios/cargo-deny/pull/840)\nresolved\n[#&#8203;797](https://redirect.github.com/EmbarkStudios/cargo-deny/issues/797)\nby passing `--filter-platform` when collecting cargo metadata if only a\nsingle target was requested either in the config or via the command\nline.\n-\n[PR#841](https://redirect.github.com/EmbarkStudios/cargo-deny/pull/841)\nfixed an issue where `--frozen` would not disable fetching of the\nadvisory DB, resolving\n[#&#8203;759](https://redirect.github.com/EmbarkStudios/cargo-deny/issues/759).\n-\n[PR#842](https://redirect.github.com/EmbarkStudios/cargo-deny/pull/842)\nand\n[PR#844](https://redirect.github.com/EmbarkStudios/cargo-deny/pull/844)\nupdated crates. Notably `krates` was updated to resolve two issues with\ncrates being pruned from the graph used when running checks. Resolving\nthese two issues may mean that updating cargo-deny may highlight issues\nthat were previously hidden.\n-\n[EmbarkStudios/krates#106](https://redirect.github.com/EmbarkStudios/krates/issues/106)\nwould fail to pull in crates brought in via a feature if that crate had\nits `lib` target renamed by the package author.\n-\n[EmbarkStudios/krates#109](https://redirect.github.com/EmbarkStudios/krates/issues/109)\nwould fail to bring in optional dependencies if they were brought in by\na weak feature in a crate *also* brought in by a weak feature.\n\n##### Changed\n\n-\n[PR#830](https://redirect.github.com/EmbarkStudios/cargo-deny/pull/830)\nremoved `gix` in favor of shelling out to `git`. This massively improves\nbuild times and eases maintenance as `gix` bumps minor versions quite\nfrequently. If cargo-deny is used in an environment that for some reason\nallows internet access but doesn't have `git` available, the advisory\ndatabase would need to be updated before calling cargo-deny.\n-\n[PR#838](https://redirect.github.com/EmbarkStudios/cargo-deny/pull/838)\nremoved `rustsec` in favor of manually implemented advisory parsing and\nchecking, with a nightly cron job that checks that the implementation\nexactly matches rustsec on the official rustsec advisory db.\n\n</details>\n\n<details>\n<summary>Swatinem/rust-cache (Swatinem/rust-cache)</summary>\n\n###\n[`v2.9.1`](https://redirect.github.com/Swatinem/rust-cache/releases/tag/v2.9.1)\n\n[Compare\nSource](https://redirect.github.com/Swatinem/rust-cache/compare/v2.9.0...v2.9.1)\n\nFix regression in hash calculation\n\n**Full Changelog**:\n<https://github.com/Swatinem/rust-cache/compare/v2.9.0...v2.9.1>\n\n###\n[`v2.9.0`](https://redirect.github.com/Swatinem/rust-cache/releases/tag/v2.9.0)\n\n[Compare\nSource](https://redirect.github.com/Swatinem/rust-cache/compare/v2.8.2...v2.9.0)\n\n##### What's Changed\n\n- Add support for running rust-cache commands from within a Nix shell by\n[@&#8203;marc0246](https://redirect.github.com/marc0246) in\n[#&#8203;290](https://redirect.github.com/Swatinem/rust-cache/pull/290)\n- Bump taiki-e/install-action from 2.62.57 to 2.62.60 in the actions\ngroup by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;291](https://redirect.github.com/Swatinem/rust-cache/pull/291)\n- Bump the actions group across 1 directory with 5 updates by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;296](https://redirect.github.com/Swatinem/rust-cache/pull/296)\n- Bump the prd-major group with 3 updates by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;294](https://redirect.github.com/Swatinem/rust-cache/pull/294)\n- Bump [@&#8203;types/node](https://redirect.github.com/types/node) from\n24.10.1 to 25.0.2 in the dev-major group by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;295](https://redirect.github.com/Swatinem/rust-cache/pull/295)\n- Consider all installed toolchains in cache key by\n[@&#8203;tamird](https://redirect.github.com/tamird) in\n[#&#8203;293](https://redirect.github.com/Swatinem/rust-cache/pull/293)\n- Compare case-insenitively for full cache key match by\n[@&#8203;kbriggs](https://redirect.github.com/kbriggs) in\n[#&#8203;303](https://redirect.github.com/Swatinem/rust-cache/pull/303)\n- Migrate to `node24` runner by\n[@&#8203;rhysd](https://redirect.github.com/rhysd) in\n[#&#8203;314](https://redirect.github.com/Swatinem/rust-cache/pull/314)\n- Bump the actions group across 1 directory with 7 updates by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;312](https://redirect.github.com/Swatinem/rust-cache/pull/312)\n- Bump the prd-minor group across 1 directory with 2 updates by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;307](https://redirect.github.com/Swatinem/rust-cache/pull/307)\n- Bump [@&#8203;types/node](https://redirect.github.com/types/node) from\n25.0.2 to 25.2.2 in the dev-minor group by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;309](https://redirect.github.com/Swatinem/rust-cache/pull/309)\n\n##### New Contributors\n\n- [@&#8203;marc0246](https://redirect.github.com/marc0246) made their\nfirst contribution in\n[#&#8203;290](https://redirect.github.com/Swatinem/rust-cache/pull/290)\n- [@&#8203;tamird](https://redirect.github.com/tamird) made their first\ncontribution in\n[#&#8203;293](https://redirect.github.com/Swatinem/rust-cache/pull/293)\n- [@&#8203;kbriggs](https://redirect.github.com/kbriggs) made their\nfirst contribution in\n[#&#8203;303](https://redirect.github.com/Swatinem/rust-cache/pull/303)\n\n**Full Changelog**:\n<https://github.com/Swatinem/rust-cache/compare/v2.8.2...v2.9.0>\n\n###\n[`v2.8.2`](https://redirect.github.com/Swatinem/rust-cache/releases/tag/v2.8.2)\n\n[Compare\nSource](https://redirect.github.com/Swatinem/rust-cache/compare/v2.8.1...v2.8.2)\n\n##### What's Changed\n\n- ci: address lint findings, add zizmor workflow by\n[@&#8203;woodruffw](https://redirect.github.com/woodruffw) in\n[#&#8203;262](https://redirect.github.com/Swatinem/rust-cache/pull/262)\n- feat: Implement ability to disable adding job ID + rust environment\nhashes to cache names by\n[@&#8203;Ryan-Brice](https://redirect.github.com/Ryan-Brice) in\n[#&#8203;279](https://redirect.github.com/Swatinem/rust-cache/pull/279)\n- Don't overwrite env for cargo-metadata call by\n[@&#8203;MaeIsBad](https://redirect.github.com/MaeIsBad) in\n[#&#8203;285](https://redirect.github.com/Swatinem/rust-cache/pull/285)\n\n##### New Contributors\n\n- [@&#8203;woodruffw](https://redirect.github.com/woodruffw) made their\nfirst contribution in\n[#&#8203;262](https://redirect.github.com/Swatinem/rust-cache/pull/262)\n- [@&#8203;Ryan-Brice](https://redirect.github.com/Ryan-Brice) made\ntheir first contribution in\n[#&#8203;279](https://redirect.github.com/Swatinem/rust-cache/pull/279)\n- [@&#8203;MaeIsBad](https://redirect.github.com/MaeIsBad) made their\nfirst contribution in\n[#&#8203;285](https://redirect.github.com/Swatinem/rust-cache/pull/285)\n\n**Full Changelog**:\n<https://github.com/Swatinem/rust-cache/compare/v2.8.1...v2.8.2>\n\n###\n[`v2.8.1`](https://redirect.github.com/Swatinem/rust-cache/releases/tag/v2.8.1)\n\n[Compare\nSource](https://redirect.github.com/Swatinem/rust-cache/compare/v2.8.0...v2.8.1)\n\n##### What's Changed\n\n- Set empty `CARGO_ENCODED_RUSTFLAGS` in workspace metadata retrieval by\n[@&#8203;ark0f](https://redirect.github.com/ark0f) in\n[#&#8203;249](https://redirect.github.com/Swatinem/rust-cache/pull/249)\n- chore(deps): update dependencies by\n[@&#8203;reneleonhardt](https://redirect.github.com/reneleonhardt) in\n[#&#8203;251](https://redirect.github.com/Swatinem/rust-cache/pull/251)\n- chore: fix dependabot groups by\n[@&#8203;reneleonhardt](https://redirect.github.com/reneleonhardt) in\n[#&#8203;253](https://redirect.github.com/Swatinem/rust-cache/pull/253)\n- Bump the prd-patch group with 2 updates by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;254](https://redirect.github.com/Swatinem/rust-cache/pull/254)\n- chore(dependabot): regenerate and commit dist/ by\n[@&#8203;reneleonhardt](https://redirect.github.com/reneleonhardt) in\n[#&#8203;257](https://redirect.github.com/Swatinem/rust-cache/pull/257)\n- Bump [@&#8203;types/node](https://redirect.github.com/types/node) from\n22.16.3 to 24.2.1 in the dev-major group by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;255](https://redirect.github.com/Swatinem/rust-cache/pull/255)\n- Bump typescript from 5.8.3 to 5.9.2 in the dev-minor group by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;256](https://redirect.github.com/Swatinem/rust-cache/pull/256)\n- Bump actions/setup-node from 4 to 5 in the actions group by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;259](https://redirect.github.com/Swatinem/rust-cache/pull/259)\n- Update README.md by\n[@&#8203;Propfend](https://redirect.github.com/Propfend) in\n[#&#8203;234](https://redirect.github.com/Swatinem/rust-cache/pull/234)\n- Bump [@&#8203;types/node](https://redirect.github.com/types/node) from\n24.2.1 to 24.3.0 in the dev-minor group by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;258](https://redirect.github.com/Swatinem/rust-cache/pull/258)\n\n##### New Contributors\n\n- [@&#8203;ark0f](https://redirect.github.com/ark0f) made their first\ncontribution in\n[#&#8203;249](https://redirect.github.com/Swatinem/rust-cache/pull/249)\n- [@&#8203;reneleonhardt](https://redirect.github.com/reneleonhardt)\nmade their first contribution in\n[#&#8203;251](https://redirect.github.com/Swatinem/rust-cache/pull/251)\n- [@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot]\nmade their first contribution in\n[#&#8203;254](https://redirect.github.com/Swatinem/rust-cache/pull/254)\n- [@&#8203;Propfend](https://redirect.github.com/Propfend) made their\nfirst contribution in\n[#&#8203;234](https://redirect.github.com/Swatinem/rust-cache/pull/234)\n\n**Full Changelog**:\n<https://github.com/Swatinem/rust-cache/compare/v2...v2.8.1>\n\n###\n[`v2.8.0`](https://redirect.github.com/Swatinem/rust-cache/releases/tag/v2.8.0)\n\n[Compare\nSource](https://redirect.github.com/Swatinem/rust-cache/compare/v2.7.8...v2.8.0)\n\n##### What's Changed\n\n- Add cache-workspace-crates feature by\n[@&#8203;jbransen](https://redirect.github.com/jbransen) in\n[#&#8203;246](https://redirect.github.com/Swatinem/rust-cache/pull/246)\n- Feat: support warpbuild cache provider by\n[@&#8203;stegaBOB](https://redirect.github.com/stegaBOB) in\n[#&#8203;247](https://redirect.github.com/Swatinem/rust-cache/pull/247)\n\n##### New Contributors\n\n- [@&#8203;jbransen](https://redirect.github.com/jbransen) made their\nfirst contribution in\n[#&#8203;246](https://redirect.github.com/Swatinem/rust-cache/pull/246)\n- [@&#8203;stegaBOB](https://redirect.github.com/stegaBOB) made their\nfirst contribution in\n[#&#8203;247](https://redirect.github.com/Swatinem/rust-cache/pull/247)\n\n**Full Changelog**:\n<https://github.com/Swatinem/rust-cache/compare/v2.7.8...v2.8.0>\n\n###\n[`v2.7.8`](https://redirect.github.com/Swatinem/rust-cache/releases/tag/v2.7.8)\n\n[Compare\nSource](https://redirect.github.com/Swatinem/rust-cache/compare/v2.7.7...v2.7.8)\n\n##### What's Changed\n\n- Include CPU arch in the cache key for arm64 Linux runners by\n[@&#8203;rhysd](https://redirect.github.com/rhysd) in\n[#&#8203;228](https://redirect.github.com/Swatinem/rust-cache/pull/228)\n\n**Full Changelog**:\n<https://github.com/Swatinem/rust-cache/compare/v2.7.7...v2.7.8>\n\n###\n[`v2.7.7`](https://redirect.github.com/Swatinem/rust-cache/releases/tag/v2.7.7)\n\n[Compare\nSource](https://redirect.github.com/Swatinem/rust-cache/compare/v2.7.6...v2.7.7)\n\n**Full Changelog**:\n<https://github.com/Swatinem/rust-cache/compare/v2.7.6...v2.7.7>\n\n###\n[`v2.7.6`](https://redirect.github.com/Swatinem/rust-cache/releases/tag/v2.7.6)\n\n[Compare\nSource](https://redirect.github.com/Swatinem/rust-cache/compare/v2.7.5...v2.7.6)\n\n##### What's Changed\n\n- Updated artifact upload action to v4 by\n[@&#8203;guylamar2006](https://redirect.github.com/guylamar2006) in\n[#&#8203;212](https://redirect.github.com/Swatinem/rust-cache/pull/212)\n- Adds an option to do lookup-only of the cache by\n[@&#8203;danlec](https://redirect.github.com/danlec) in\n[#&#8203;217](https://redirect.github.com/Swatinem/rust-cache/pull/217)\n- add runner OS in cache key by\n[@&#8203;rnbguy](https://redirect.github.com/rnbguy) in\n[#&#8203;220](https://redirect.github.com/Swatinem/rust-cache/pull/220)\n- Allow opting out of caching $CARGO\\_HOME/bin. by\n[@&#8203;benjyw](https://redirect.github.com/benjyw) in\n[#&#8203;216](https://redirect.github.com/Swatinem/rust-cache/pull/216)\n\n##### New Contributors\n\n- [@&#8203;guylamar2006](https://redirect.github.com/guylamar2006) made\ntheir first contribution in\n[#&#8203;212](https://redirect.github.com/Swatinem/rust-cache/pull/212)\n- [@&#8203;danlec](https://redirect.github.com/danlec) made their first\ncontribution in\n[#&#8203;217](https://redirect.github.com/Swatinem/rust-cache/pull/217)\n- [@&#8203;rnbguy](https://redirect.github.com/rnbguy) made their first\ncontribution in\n[#&#8203;220](https://redirect.github.com/Swatinem/rust-cache/pull/220)\n- [@&#8203;benjyw](https://redirect.github.com/benjyw) made their first\ncontribution in\n[#&#8203;216](https://redirect.github.com/Swatinem/rust-cache/pull/216)\n\n**Full Changelog**:\n<https://github.com/Swatinem/rust-cache/compare/v2.7.5...v2.7.6>\n\n###\n[`v2.7.5`](https://redirect.github.com/Swatinem/rust-cache/releases/tag/v2.7.5)\n\n[Compare\nSource](https://redirect.github.com/Swatinem/rust-cache/compare/v2.7.3...v2.7.5)\n\n##### What's Changed\n\n- Upgrade checkout action from version 3 to 4 by\n[@&#8203;carsten-wenderdel](https://redirect.github.com/carsten-wenderdel)\nin\n[#&#8203;190](https://redirect.github.com/Swatinem/rust-cache/pull/190)\n- fix: usage of `deprecated` version of `node` by\n[@&#8203;hamirmahal](https://redirect.github.com/hamirmahal) in\n[#&#8203;197](https://redirect.github.com/Swatinem/rust-cache/pull/197)\n- Only run macOsWorkaround() on macOS by\n[@&#8203;heksesang](https://redirect.github.com/heksesang) in\n[#&#8203;206](https://redirect.github.com/Swatinem/rust-cache/pull/206)\n- Support Cargo.lock format cargo-lock v4 by\n[@&#8203;NobodyXu](https://redirect.github.com/NobodyXu) in\n[#&#8203;211](https://redirect.github.com/Swatinem/rust-cache/pull/211)\n\n##### New Contributors\n\n-\n[@&#8203;carsten-wenderdel](https://redirect.github.com/carsten-wenderdel)\nmade their first contribution in\n[#&#8203;190](https://redirect.github.com/Swatinem/rust-cache/pull/190)\n- [@&#8203;hamirmahal](https://redirect.github.com/hamirmahal) made\ntheir first contribution in\n[#&#8203;197](https://redirect.github.com/Swatinem/rust-cache/pull/197)\n- [@&#8203;heksesang](https://redirect.github.com/heksesang) made their\nfirst contribution in\n[#&#8203;206](https://redirect.github.com/Swatinem/rust-cache/pull/206)\n\n**Full Changelog**:\n<https://github.com/Swatinem/rust-cache/compare/v2.7.3...v2.7.5>\n\n###\n[`v2.7.3`](https://redirect.github.com/Swatinem/rust-cache/releases/tag/v2.7.3)\n\n[Compare\nSource](https://redirect.github.com/Swatinem/rust-cache/compare/v2.7.2...v2.7.3)\n\n- Work around upstream problem that causes cache saving to hang for\nminutes.\n\n**Full Changelog**:\n<https://github.com/Swatinem/rust-cache/compare/v2.7.2...v2.7.3>\n\n###\n[`v2.7.2`](https://redirect.github.com/Swatinem/rust-cache/releases/tag/v2.7.2)\n\n[Compare\nSource](https://redirect.github.com/Swatinem/rust-cache/compare/v2.7.1...v2.7.2)\n\n##### What's Changed\n\n- Update action runtime to `node20` by\n[@&#8203;rhysd](https://redirect.github.com/rhysd) in\n[#&#8203;175](https://redirect.github.com/Swatinem/rust-cache/pull/175)\n- Only key by `Cargo.toml` and `Cargo.lock` files of workspace members\nby [@&#8203;max-heller](https://redirect.github.com/max-heller) in\n[#&#8203;180](https://redirect.github.com/Swatinem/rust-cache/pull/180)\n\n##### New Contributors\n\n- [@&#8203;rhysd](https://redirect.github.com/rhysd) made their first\ncontribution in\n[#&#8203;175](https://redirect.github.com/Swatinem/rust-cache/pull/175)\n- [@&#8203;max-heller](https://redirect.github.com/max-heller) made\ntheir first contribution in\n[#&#8203;180](https://redirect.github.com/Swatinem/rust-cache/pull/180)\n\n**Full Changelog**:\n<https://github.com/Swatinem/rust-cache/compare/v2.7.1...v2.7.2>\n\n###\n[`v2.7.1`](https://redirect.github.com/Swatinem/rust-cache/compare/v2.7.0...v2.7.1)\n\n[Compare\nSource](https://redirect.github.com/Swatinem/rust-cache/compare/v2.7.0...v2.7.1)\n\n###\n[`v2.7.0`](https://redirect.github.com/Swatinem/rust-cache/releases/tag/v2.7.0)\n\n[Compare\nSource](https://redirect.github.com/Swatinem/rust-cache/compare/v2.6.2...v2.7.0)\n\n##### What's Changed\n\n- Fix save-if documentation in readme by\n[@&#8203;rukai](https://redirect.github.com/rukai) in\n[#&#8203;166](https://redirect.github.com/Swatinem/rust-cache/pull/166)\n- Support for `trybuild` and similar macro testing tools by\n[@&#8203;neysofu](https://redirect.github.com/neysofu) in\n[#&#8203;168](https://redirect.github.com/Swatinem/rust-cache/pull/168)\n\n##### New Contributors\n\n- [@&#8203;rukai](https://redirect.github.com/rukai) made their first\ncontribution in\n[#&#8203;166](https://redirect.github.com/Swatinem/rust-cache/pull/166)\n- [@&#8203;neysofu](https://redirect.github.com/neysofu) made their\nfirst contribution in\n[#&#8203;168](https://redirect.github.com/Swatinem/rust-cache/pull/168)\n\n**Full Changelog**:\n<https://github.com/Swatinem/rust-cache/compare/v2.6.2...v2.7.0>\n\n###\n[`v2.6.2`](https://redirect.github.com/Swatinem/rust-cache/releases/tag/v2.6.2)\n\n[Compare\nSource](https://redirect.github.com/Swatinem/rust-cache/compare/v2.6.1...v2.6.2)\n\n##### What's Changed\n\n- dep: Use `smol-toml` instead of `toml` by\n[@&#8203;NobodyXu](https://redirect.github.com/NobodyXu) in\n[#&#8203;164](https://redirect.github.com/Swatinem/rust-cache/pull/164)\n\n**Full Changelog**:\n<https://github.com/Swatinem/rust-cache/compare/v2...v2.6.2>\n\n###\n[`v2.6.1`](https://redirect.github.com/Swatinem/rust-cache/releases/tag/v2.6.1)\n\n[Compare\nSource](https://redirect.github.com/Swatinem/rust-cache/compare/v2.6.0...v2.6.1)\n\n- Fix hash contributions of `Cargo.lock`/`Cargo.toml` files.\n\n###\n[`v2.6.0`](https://redirect.github.com/Swatinem/rust-cache/releases/tag/v2.6.0)\n\n[Compare\nSource](https://redirect.github.com/Swatinem/rust-cache/compare/v2.5.1...v2.6.0)\n\n##### What's Changed\n\n- Add \"buildjet\" as a second `cache-provider` backend\n[@&#8203;joroshiba](https://redirect.github.com/joroshiba) in\n[#&#8203;154](https://redirect.github.com/Swatinem/rust-cache/pull/154)\n- Clean up sparse registry index.\n- Do not clean up src of `-sys` crates.\n- Remove `.cargo/credentials.toml` before saving.\n\n##### New Contributors\n\n- [@&#8203;joroshiba](https://redirect.github.com/joroshiba) made their\nfirst contribution in\n[#&#8203;154](https://redirect.github.com/Swatinem/rust-cache/pull/154)\n\n**Full Changelog**:\n<https://github.com/Swatinem/rust-cache/compare/v2.5.1...v2.6.0>\n\n###\n[`v2.5.1`](https://redirect.github.com/Swatinem/rust-cache/releases/tag/v2.5.1)\n\n[Compare\nSource](https://redirect.github.com/Swatinem/rust-cache/compare/v2.5.0...v2.5.1)\n\n- Fix hash contribution of `Cargo.lock`.\n\n###\n[`v2.5.0`](https://redirect.github.com/Swatinem/rust-cache/releases/tag/v2.5.0)\n\n[Compare\nSource](https://redirect.github.com/Swatinem/rust-cache/compare/v2.4.0...v2.5.0)\n\n##### What's Changed\n\n- feat: Rm workspace crates version before caching by\n[@&#8203;NobodyXu](https://redirect.github.com/NobodyXu) in\n[#&#8203;147](https://redirect.github.com/Swatinem/rust-cache/pull/147)\n- feat: Add hash of `.cargo/config.toml` to key by\n[@&#8203;NobodyXu](https://redirect.github.com/NobodyXu) in\n[#&#8203;149](https://redirect.github.com/Swatinem/rust-cache/pull/149)\n\n##### New Contributors\n\n- [@&#8203;NobodyXu](https://redirect.github.com/NobodyXu) made their\nfirst contribution in\n[#&#8203;147](https://redirect.github.com/Swatinem/rust-cache/pull/147)\n\n**Full Changelog**:\n<https://github.com/Swatinem/rust-cache/compare/v2.4.0...v2.5.0>\n\n###\n[`v2.4.0`](https://redirect.github.com/Swatinem/rust-cache/releases/tag/v2.4.0)\n\n[Compare\nSource](https://redirect.github.com/Swatinem/rust-cache/compare/v2.3.0...v2.4.0)\n\n- Fix cache key stability.\n- Use 8 character hash components to reduce the key length, making it\nmore readable.\n\n###\n[`v2.3.0`](https://redirect.github.com/Swatinem/rust-cache/releases/tag/v2.3.0)\n\n[Compare\nSource](https://redirect.github.com/Swatinem/rust-cache/compare/v2.2.1...v2.3.0)\n\n- Add `cache-all-crates` option, which enables caching of crates\ninstalled by workflows.\n- Add installed packages to cache key, so changes to workflows that\ninstall rust tools are detected and cached properly.\n- Fix cache restore failures due to upstream bug.\n- Fix `EISDIR` error due to globed directories.\n- Update runtime `@actions/cache`, `@actions/io` and dev `typescript`\ndependencies.\n- Update `npm run prepare` so it creates distribution files with the\nright line endings.\n\n###\n[`v2.2.1`](https://redirect.github.com/Swatinem/rust-cache/releases/tag/v2.2.1)\n\n[Compare\nSource](https://redirect.github.com/Swatinem/rust-cache/compare/v2.2.0...v2.2.1)\n\n- Update `@actions/cache` dependency to fix usage of `zstd` compression.\n\n###\n[`v2.2.0`](https://redirect.github.com/Swatinem/rust-cache/releases/tag/v2.2.0)\n\n[Compare\nSource](https://redirect.github.com/Swatinem/rust-cache/compare/v2.1.0...v2.2.0)\n\n- Add new `save-if` option to always restore, but only conditionally\nsave the cache.\n\n###\n[`v2.1.0`](https://redirect.github.com/Swatinem/rust-cache/releases/tag/v2.1.0)\n\n[Compare\nSource](https://redirect.github.com/Swatinem/rust-cache/compare/v2.0.2...v2.1.0)\n\n- Only hash `Cargo.{lock,toml}` files in the configured workspace\ndirectories.\n\n###\n[`v2.0.2`](https://redirect.github.com/Swatinem/rust-cache/releases/tag/v2.0.2)\n\n[Compare\nSource](https://redirect.github.com/Swatinem/rust-cache/compare/v2.0.1...v2.0.2)\n\n- Avoid calling cargo metadata on pre-cleanup.\n- Added `prefix-key`, `cache-directories` and `cache-targets` options.\n\n###\n[`v2.0.1`](https://redirect.github.com/Swatinem/rust-cache/releases/tag/v2.0.1)\n\n[Compare\nSource](https://redirect.github.com/Swatinem/rust-cache/compare/v2...v2.0.1)\n\n- Primarily just updating dependencies to fix GitHub deprecation\nnotices.\n\n</details>\n\n<details>\n<summary>actions/create-github-app-token\n(actions/create-github-app-token)</summary>\n\n###\n[`v3.1.1`](https://redirect.github.com/actions/create-github-app-token/releases/tag/v3.1.1)\n\n[Compare\nSource](https://redirect.github.com/actions/create-github-app-token/compare/v3.1.0...v3.1.1)\n\n##### Bug Fixes\n\n- improve error message when app identifier is empty\n([#&#8203;362](https://redirect.github.com/actions/create-github-app-token/issues/362))\n([07e2b76](https://redirect.github.com/actions/create-github-app-token/commit/07e2b760664f080c40eec4eacf7477256582db36)),\ncloses\n[#&#8203;249](https://redirect.github.com/actions/create-github-app-token/issues/249)\n\n###\n[`v3.1.0`](https://redirect.github.com/actions/create-github-app-token/releases/tag/v3.1.0)\n\n[Compare\nSource](https://redirect.github.com/actions/create-github-app-token/compare/v3...v3.1.0)\n\n##### Bug Fixes\n\n- **deps:** bump p-retry from 7.1.1 to 8.0.0\n([#&#8203;357](https://redirect.github.com/actions/create-github-app-token/issues/357))\n([3bbe07d](https://redirect.github.com/actions/create-github-app-token/commit/3bbe07d928e2d6c30bf3e37c6b89edbc4045facf))\n\n##### Features\n\n- add `client-id` input and deprecate `app-id`\n([#&#8203;353](https://redirect.github.com/actions/create-github-app-token/issues/353))\n([e6bd4e6](https://redirect.github.com/actions/create-github-app-token/commit/e6bd4e6970172bed9fe138b2eaf4cbffa4cca8f9))\n- update permission inputs\n([#&#8203;358](https://redirect.github.com/actions/create-github-app-token/issues/358))\n([076e948](https://redirect.github.com/actions/create-github-app-token/commit/076e9480ca6e9633bff412d05eff0fc2f1e7d2be))\n\n</details>\n\n<details>\n<summary>actions/setup-node (actions/setup-node)</summary>\n\n###\n[`v6.4.0`](https://redirect.github.com/actions/setup-node/compare/v6.3.0...v6.4.0)\n\n[Compare\nSource](https://redirect.github.com/actions/setup-node/compare/v6.3.0...v6.4.0)\n\n</details>\n\n<details>\n<summary>actions/upload-artifact (actions/upload-artifact)</summary>\n\n###\n[`v7.0.1`](https://redirect.github.com/actions/upload-artifact/releases/tag/v7.0.1)\n\n[Compare\nSource](https://redirect.github.com/actions/upload-artifact/compare/v7...v7.0.1)\n\n##### What's Changed\n\n- Update the readme with direct upload details by\n[@&#8203;danwkennedy](https://redirect.github.com/danwkennedy) in\n[#&#8203;795](https://redirect.github.com/actions/upload-artifact/pull/795)\n- Readme: bump all the example versions to v7 by\n[@&#8203;danwkennedy](https://redirect.github.com/danwkennedy) in\n[#&#8203;796](https://redirect.github.com/actions/upload-artifact/pull/796)\n- Include changes in typespec/ts-http-runtime 0.3.5 by\n[@&#8203;yacaovsnc](https://redirect.github.com/yacaovsnc) in\n[#&#8203;797](https://redirect.github.com/actions/upload-artifact/pull/797)\n\n**Full Changelog**:\n<https://github.com/actions/upload-artifact/compare/v7...v7.0.1>\n\n</details>\n\n<details>\n<summary>github/codeql-action (github/codeql-action)</summary>\n\n###\n[`v4.35.3`](https://redirect.github.com/github/codeql-action/releases/tag/v4.35.3)\n\n[Compare\nSource](https://redirect.github.com/github/codeql-action/compare/v4.35.2...v4.35.3)\n\n- *Upcoming breaking change*: Add a deprecation warning for customers\nusing CodeQL version 2.19.3 and earlier. These versions of CodeQL were\ndiscontinued on 9 April 2026 alongside GitHub Enterprise Server 3.15,\nand will be unsupported by the next minor release of the CodeQL Action.\n[#&#8203;3837](https://redirect.github.com/github/codeql-action/pull/3837)\n- Configurations for private registries that use Cloudsmith or GCP OIDC\nare now accepted.\n[#&#8203;3850](https://redirect.github.com/github/codeql-action/pull/3850)\n- Best-effort connection tests for private registries now use `GET`\nrequests instead of `HEAD` for better compatibility with various\nregistry implementations. For NuGet feeds, the test is now always\nperformed against the service index.\n[#&#8203;3853](https://redirect.github.com/github/codeql-action/pull/3853)\n- Fixed a bug where two diagnostics produced within the same millisecond\ncould overwrite each other on disk, causing one of them to be lost.\n[#&#8203;3852](https://redirect.github.com/github/codeql-action/pull/3852)\n- Update default CodeQL bundle version to\n[2.25.3](https://redirect.github.com/github/codeql-action/releases/tag/codeql-bundle-v2.25.3).\n[#&#8203;3865](https://redirect.github.com/github/codeql-action/pull/3865)\n\n###\n[`v4.35.2`](https://redirect.github.com/github/codeql-action/releases/tag/v4.35.2)\n\n[Compare\nSource](https://redirect.github.com/github/codeql-action/compare/v4.35.1...v4.35.2)\n\n- The undocumented TRAP cache cleanup feature that could be enabled\nusing the `CODEQL_ACTION_CLEANUP_TRAP_CACHES` environment variable is\ndeprecated and will be removed in May 2026. If you are affected by this,\nwe recommend disabling TRAP caching by passing the `trap-caching: false`\ninput to the `init` Action.\n[#&#8203;3795](https://redirect.github.com/github/codeql-action/pull/3795)\n- The Git version 2.36.0 requirement for improved incremental analysis\nnow only applies to repositories that contain submodules.\n[#&#8203;3789](https://redirect.github.com/github/codeql-action/pull/3789)\n- Python analysis on GHES no longer extracts the standard library,\nrelying instead on models of the standard library. This should result in\nsignificantly faster extraction and analysis times, while the effect on\nalerts should be minimal.\n[#&#8203;3794](https://redirect.github.com/github/codeql-action/pull/3794)\n- Fixed a bug in the validation of OIDC configurations for private\nregistries that was added in CodeQL Action 4.33.0 / 3.33.0.\n[#&#8203;3807](https://redirect.github.com/github/codeql-action/pull/3807)\n- Update default CodeQL bundle version to\n[2.25.2](https://redirect.github.com/github/codeql-action/releases/tag/codeql-bundle-v2.25.2).\n[#&#8203;3823](https://redirect.github.com/github/codeql-action/pull/3823)\n\n</details>\n\n<details>\n<summary>actions/go-versions (go)</summary>\n\n###\n[`v1.26.2`](https://redirect.github.com/actions/go-versions/releases/tag/1.26.2-24114135105):\n1.26.2\n\n[Compare\nSource](https://redirect.github.com/actions/go-versions/compare/1.26.1-22746851271...1.26.2-24114135105)\n\nGo 1.26.2\n\n</details>\n\n<details>\n<summary>step-security/harden-runner\n(step-security/harden-runner)</summary>\n\n###\n[`v2.19.0`](https://redirect.github.com/step-security/harden-runner/releases/tag/v2.19.0)\n\n[Compare\nSource](https://redirect.github.com/step-security/harden-runner/compare/v2.18.0...v2.19.0)\n\n##### What's Changed\n\n##### New Runner Support\n\nHarden-Runner now supports Depot, Blacksmith, Namespace, and WarpBuild\nrunners with the same egress monitoring, runtime monitoring, and policy\nenforcement available on GitHub-hosted runners.\n\n##### Automated Incident Response for Supply Chain Attacks\n\n- Global block list: Outbound connections to known malicious domains and\nIPs are now blocked even in audit mode.\n- System-defined detection rules: Harden-Runner will trigger lockdown\nmode when a high risk event is detected during an active supply chain\nattack (for example, a process reading the memory of the runner worker\nprocess, a common technique for stealing GitHub Actions secrets).\n\n##### Bug Fixes\n\nWindows and macOS: stability and reliability fixes\n\n**Full Changelog**:\n<https://github.com/step-security/harden-runner/compare/v2.18.0...v2.19.0>\n\n###\n[`v2.18.0`](https://redirect.github.com/step-security/harden-runner/releases/tag/v2.18.0)\n\n[Compare\nSource](https://redirect.github.com/step-security/harden-runner/compare/v2.17.0...v2.18.0)\n\n##### What's Changed\n\nGlobal Block List: During supply chain incidents like the recent axios\nand trivy compromises, StepSecurity will add known malicious domains and\nIP addresses (IOCs) to a global block list. These will be automatically\nblocked, even in audit mode, providing immediate protection without\nrequiring any workflow changes.\n\nDeploy on Self-Hosted VM: Added `deploy-on-self-hosted-vm` input that\nallows the Harden Runner agent to be installed directly on ephemeral\nself-hosted Linux runner VMs at workflow runtime. This is intended as an\nalternative when baking the agent into the VM image is not possible.\n\n**Full Changelog**:\n<https://github.com/step-security/harden-runner/compare/v2.17.0...v2.18.0>\n\n###\n[`v2.17.0`](https://redirect.github.com/step-security/harden-runner/releases/tag/v2.17.0)\n\n[Compare\nSource](https://redirect.github.com/step-security/harden-runner/compare/v2.16.1...v2.17.0)\n\n##### What's Changed\n\n##### Policy Store Support\n\nAdded `use-policy-store` and `api-key` inputs to fetch security policies\ndirectly from the [StepSecurity Policy\nStore](https://docs.stepsecurity.io/harden-runner/policy-store).\nPolicies can be defined and attached at the workflow, repo, org, or\ncluster (ARC) level, with the most granular policy taking precedence.\nThis is the preferred method over the existing `policy` input which\nrequires `id-token: write` permission. If no policy is found in the\nstore, the action defaults to audit mode.\n\n**Full Changelog**:\n<https://github.com/step-security/harden-runner/compare/v2.16.1...v2.17.0>\n\n</details>\n\n<details>\n<summary>taiki-e/install-action (taiki-e/install-action)</summary>\n\n###\n[`v2.75.28`](https://redirect.github.com/taiki-e/install-action/compare/v2.75.27...v2.75.28)\n\n[Compare\nSource](https://redirect.github.com/taiki-e/install-action/compare/v2.75.27...v2.75.28)\n\n###\n[`v2.75.27`](https://redirect.github.com/taiki-e/install-action/releases/tag/v2.75.27):\n2.75.27\n\n[Compare\nSource](https://redirect.github.com/taiki-e/install-action/compare/v2.75.26...v2.75.27)\n\n- Update `cargo-udeps@latest` to 0.1.61.\n\n- Update `wasm-tools@latest` to 1.248.0.\n\n- Update `cargo-deb@latest` to 3.6.4.\n\n###\n[`v2.75.26`](https://redirect.github.com/taiki-e/install-action/releases/tag/v2.75.26):\n2.75.26\n\n[Compare\nSource](https://redirect.github.com/taiki-e/install-action/compare/v2.75.25...v2.75.26)\n\n- Update `wasm-bindgen@latest` to 0.2.120.\n\n- Update `mise@latest` to 2026.4.25.\n\n- Update `martin@latest` to 1.8.0.\n\n- Update `vacuum@latest` to 0.26.4.\n\n###\n[`v2.75.25`](https://redirect.github.com/taiki-e/install-action/releases/tag/v2.75.25):\n2.75.25\n\n[Compare\nSource](https://redirect.github.com/taiki-e/install-action/compare/v2.75.24...v2.75.25)\n\n- Update `uv@latest` to 0.11.8.\n\n- Update `typos@latest` to 1.45.2.\n\n- Update `tombi@latest` to 0.9.25.\n\n- Update `mise@latest` to 2026.4.24.\n\n###\n[`v2.75.24`](https://redirect.github.com/taiki-e/install-action/releases/tag/v2.75.24):\n2.75.24\n\n[Compare\nSource](https://redirect.github.com/taiki-e/install-action/compare/v2.75.23...v2.75.24)\n\n- Update `prek@latest` to 0.3.11.\n\n- Update `mise@latest` to 2026.4.23.\n\n- Update `vacuum@latest` to 0.26.3.\n\n###\n[`v2.75.23`](https://redirect.github.com/taiki-e/install-action/releases/tag/v2.75.23):\n2.75.23\n\n[Compare\nSource](https://redirect.github.com/taiki-e/install-action/compare/v2.75.22...v2.75.23)\n\n- Update `vacuum@latest` to 0.26.2.\n\n- Update `tombi@latest` to 0.9.24.\n\n- Update `mise@latest` to 2026.4.22.\n\n- Update `martin@latest` to 1.7.0.\n\n- Update `git-cliff@latest` to 2.13.1.\n\n- Update `cargo-tarpaulin@latest` to 0.35.4.\n\n- Update `cargo-sort@latest` to 2.1.4.\n\n###\n[`v2.75.22`](https://redirect.github.com/taiki-e/install-action/releases/tag/v2.75.22):\n2.75.22\n\n[Compare\nSource](https://redirect.github.com/taiki-e/install-action/compare/v2.75.21...v2.75.22)\n\n- Update `tombi@latest` to 0.9.22.\n\n- Update `biome@latest` to 2.4.13.\n\n###\n[`v2.75.21`](https://redirect.github.com/taiki-e/install-action/releases/tag/v2.75.21):\n2.75.21\n\n[Compare\nSource](https://redirect.github.com/taiki-e/install-action/compare/v2.75.20...v2.75.21)\n\n- Update `mise@latest` to 2026.4.19.\n\n- Update `tombi@latest` to 0.9.21.\n\n- Update `syft@latest` to 1.43.0.\n\n###\n[`v2.75.20`](https://redirect.github.com/taiki-e/install-action/releases/tag/v2.75.20):\n2.75.20\n\n[Compare\nSource](https://redirect.github.com/taiki-e/install-action/compare/v2.75.19...v2.75.20)\n\n- Update `prek@latest` to 0.3.10.\n\n- Update `cargo-xwin@latest` to 0.22.0.\n\n###\n[`v2.75.19`](https://redirect.github.com/taiki-e/install-action/releases/tag/v2.75.19):\n2.75.19\n\n[Compare\nSource](https://redirect.github.com/taiki-e/install-action/compare/v2.75.18...v2.75.19)\n\n- Update `wasmtime@latest` to 44.0.0.\n\n- Update `tombi@latest` to 0.9.20.\n\n- Update `martin@latest` to 1.6.0.\n\n- Update `just@latest` to 1.50.0.\n\n- Update `mise@latest` to 2026.4.18.\n\n- Update `rclone@latest` to 1.73.5.\n\n###\n[`v2.75.18`](https://redirect.github.com/taiki-e/install-action/releases/tag/v2.75.18):\n2.75.18\n\n[Compare\nSource](https://redirect.github.com/taiki-e/install-action/compare/v2.75.17...v2.75.18)\n\n- Update `vacuum@latest` to 0.26.1.\n\n- Update `wasm-tools@latest` to 1.247.0.\n\n- Update `mise@latest` to 2026.4.16.\n\n- Update `espup@latest` to 0.17.1.\n\n- Update `trivy@latest` to 0.70.0.\n\n###\n[`v2.75.17`](https://redirect.github.com/taiki-e/install-action/blob/HEAD/CHANGELOG.md#100---2021-12-30)\n\n[Compare\nSource](https://redirect.github.com/taiki-e/install-action/compare/v2.75.16...v2.75.17)\n\nInitial release\n\n[Unreleased]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.75.21...HEAD\n\n[2.75.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.75.20...v2.75.21\n\n[2.75.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.75.19...v2.75.20\n\n[2.75.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.75.18...v2.75.19\n\n[2.75.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.75.17...v2.75.18\n\n[2.75.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.75.16...v2.75.17\n\n[2.75.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.75.15...v2.75.16\n\n[2.75.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.75.14...v2.75.15\n\n[2.75.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.75.13...v2.75.14\n\n[2.75.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.75.12...v2.75.13\n\n[2.75.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.75.11...v2.75.12\n\n[2.75.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.75.10...v2.75.11\n\n[2.75.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.75.9...v2.75.10\n\n[2.75.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.75.8...v2.75.9\n\n[2.75.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.75.7...v2.75.8\n\n[2.75.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.75.6...v2.75.7\n\n[2.75.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.75.5...v2.75.6\n\n[2.75.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.75.4...v2.75.5\n\n[2.75.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.75.3...v2.75.4\n\n[2.75.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.75.2...v2.75.3\n\n[2.75.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.75.1...v2.75.2\n\n[2.75.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.75.0...v2.75.1\n\n[2.75.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.74.1...v2.75.0\n\n[2.74.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.74.0...v2.74.1\n\n[2.74.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.73.0...v2.74.0\n\n[2.73.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.72.0...v2.73.0\n\n[2.72.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.71.3...v2.72.0\n\n[2.71.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.71.2...v2.71.3\n\n[2.71.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.71.1...v2.71.2\n\n[2.71.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.71.0...v2.71.1\n\n[2.71.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.70.4...v2.71.0\n\n[2.70.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.70.3...v2.70.4\n\n[2.70.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.70.2...v2.70.3\n\n[2.70.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.70.1...v2.70.2\n\n[2.70.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.70.0...v2.70.1\n\n[2.70.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.69.14...v2.70.0\n\n[2.69.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.69.13...v2.69.14\n\n[2.69.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.69.12...v2.69.13\n\n[2.69.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.69.11...v2.69.12\n\n[2.69.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.69.10...v2.69.11\n\n[2.69.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.69.9...v2.69.10\n\n[2.69.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.69.8...v2.69.9\n\n[2.69.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.69.7...v2.69.8\n\n[2.69.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.69.6...v2.69.7\n\n[2.69.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.69.5...v2.69.6\n\n[2.69.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.69.4...v2.69.5\n\n[2.69.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.69.3...v2.69.4\n\n[2.69.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.69.2...v2.69.3\n\n[2.69.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.69.1...v2.69.2\n\n[2.69.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.69.0...v2.69.1\n\n[2.69.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.36...v2.69.0\n\n[2.68.36]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.35...v2.68.36\n\n[2.68.35]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.34...v2.68.35\n\n[2.68.34]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.33...v2.68.34\n\n[2.68.33]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.32...v2.68.33\n\n[2.68.32]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.31...v2.68.32\n\n[2.68.31]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.30...v2.68.31\n\n[2.68.30]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.29...v2.68.30\n\n[2.68.29]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.28...v2.68.29\n\n[2.68.28]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.27...v2.68.28\n\n[2.68.27]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.26...v2.68.27\n\n[2.68.26]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.25...v2.68.26\n\n[2.68.25]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.24...v2.68.25\n\n[2.68.24]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.23...v2.68.24\n\n[2.68.23]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.22...v2.68.23\n\n[2.68.22]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.21...v2.68.22\n\n[2.68.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.20...v2.68.21\n\n[2.68.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.19...v2.68.20\n\n[2.68.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.18...v2.68.19\n\n[2.68.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.17...v2.68.18\n\n[2.68.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.16...v2.68.17\n\n[2.68.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.15...v2.68.16\n\n[2.68.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.14...v2.68.15\n\n[2.68.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.13...v2.68.14\n\n[2.68.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.12...v2.68.13\n\n[2.68.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.11...v2.68.12\n\n[2.68.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.10...v2.68.11\n\n[2.68.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.9...v2.68.10\n\n[2.68.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.8...v2.68.9\n\n[2.68.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.7...v2.68.8\n\n[2.68.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.6...v2.68.7\n\n[2.68.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.5...v2.68.6\n\n[2.68.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.4...v2.68.5\n\n[2.68.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.3...v2.68.4\n\n[2.68.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.2...v2.68.3\n\n[2.68.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.1...v2.68.2\n\n[2.68.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.0...v2.68.1\n\n[2.68.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.30...v2.68.0\n\n[2.67.30]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.29...v2.67.30\n\n[2.67.29]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.28...v2.67.29\n\n[2.67.28]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.27...v2.67.28\n\n[2.67.27]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.26...v2.67.27\n\n[2.67.26]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.25...v2.67.26\n\n[2.67.25]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.24...v2.67.25\n\n[2.67.24]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.23...v2.67.24\n\n[2.67.23]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.22...v2.67.23\n\n[2.67.22]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.21...v2.67.22\n\n[2.67.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.20...v2.67.21\n\n[2.67.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.19...v2.67.20\n\n[2.67.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.18...v2.67.19\n\n[2.67.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.17...v2.67.18\n\n[2.67.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.16...v2.67.17\n\n[2.67.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.15...v2.67.16\n\n[2.67.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.14...v2.67.15\n\n[2.67.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.13...v2.67.14\n\n[2.67.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.12...v2.67.13\n\n[2.67.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.11...v2.67.12\n\n[2.67.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.10...v2.67.11\n\n[2.67.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.9...v2.67.10\n\n[2.67.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.8...v2.67.9\n\n[2.67.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.7...v2.67.8\n\n[2.67.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.6...v2.67.7\n\n[2.67.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.5...v2.67.6\n\n[2.67.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.4...v2.67.5\n\n[2.67.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.3...v2.67.4\n\n[2.67.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.2...v2.67.3\n\n[2.67.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.1...v2.67.2\n\n[2.67.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.0...v2.67.1\n\n[2.67.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.66.7...v2.67.0\n\n[2.66.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.66.6...v2.66.7\n\n[2.66.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.66.5...v2.66.6\n\n[2.66.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.66.4...v2.66.5\n\n[2.66.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.66.3...v2.66.4\n\n[2.66.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.66.2...v2.66.3\n\n[2.66.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.66.1...v2.66.2\n\n[2.66.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.66.0...v2.66.1\n\n[2.66.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.16...v2.66.0\n\n[2.65.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.15...v2.65.16\n\n[2.65.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.14...v2.65.15\n\n[2.65.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.13...v2.65.14\n\n[2.65.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.12...v2.65.13\n\n[2.65.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.11...v2.65.12\n\n[2.65.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.10...v2.65.11\n\n[2.65.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.9...v2.65.10\n\n[2.65.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.8...v2.65.9\n\n[2.65.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.7...v2.65.8\n\n[2.65.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.6...v2.65.7\n\n[2.65.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.5...v2.65.6\n\n[2.65.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.4...v2.65.5\n\n[2.65.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.3...v2.65.4\n\n[2.65.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.2...v2.65.3\n\n[2.65.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.1...v2.65.2\n\n[2.65.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.0...v2.65.1\n\n[2.65.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.64.2...v2.65.0\n\n[2.64.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.64.1...v2.64.2\n\n[2.64.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.64.0...v2.64.1\n\n[2.64.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.63.3...v2.64.0\n\n[2.63.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.63.2...v2.63.3\n\n[2.63.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.63.1...v2.63.2\n\n[2.63.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.63.0...v2.63.1\n\n[2.63.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.67...v2.63.0\n\n[2.62.67]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.66...v2.62.67\n\n[2.62.66]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.65...v2.62.66\n\n[2.62.65]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.64...v2.62.65\n\n[2.62.64]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.63...v2.62.64\n\n[2.62.63]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.62...v2.62.63\n\n[2.62.62]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.61...v2.62.62\n\n[2.62.61]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.60...v2.62.61\n\n[2.62.60]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.59...v2.62.60\n\n[2.62.59]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.58...v2.62.59\n\n[2.62.58]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.57...v2.62.58\n\n[2.62.57]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.56...v2.62.57\n\n[2.62.56]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.55...v2.62.56\n\n[2.62.55]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.54...v2.62.55\n\n[2.62.54]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.53...v2.62.54\n\n[2.62.53]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.52...v2.62.53\n\n[2.62.52]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.51...v2.62.52\n\n[2.62.51]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.50...v2.62.51\n\n[2.62.50]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.49...v2.62.50\n\n[2.62.49]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.48...v2.62.49\n\n[2.62.48]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.47...v2.62.48\n\n[2.62.47]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.46...v2.62.47\n\n[2.62.46]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.45...v2.62.46\n\n[2.62.45]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.44...v2.62.45\n\n[2.62.44]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.43...v2.62.44\n\n[2.62.43]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.42...v2.62.43\n\n[2.62.42]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.41...v2.62.42\n\n[2.62.41]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.40...v2.62.41\n\n[2.62.40]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.39...v2.62.40\n\n[2.62.39]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.38...v2.62.39\n\n[2.62.38]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.37...v2.62.38\n\n[2.62.37]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.36...v2.62.37\n\n[2.62.36]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.35...v2.62.36\n\n[2.62.35]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.34...v2.62.35\n\n[2.62.34]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.33...v2.62.34\n\n[2.62.33]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.32...v2.62.33\n\n[2.62.32]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.31...v2.62.32\n\n[2.62.31]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.30...v2.62.31\n\n[2.62.30]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.29...v2.62.30\n\n[2.62.29]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.28...v2.62.29\n\n[2.62.28]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.27...v2.62.28\n\n[2.62.27]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.26...v2.62.27\n\n[2.62.26]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.25...v2.62.26\n\n[2.62.25]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.24...v2.62.25\n\n[2.62.24]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.23...v2.62.24\n\n[2.62.23]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.22...v2.62.23\n\n[2.62.22]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.21...v2.62.22\n\n[2.62.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.20...v2.62.21\n\n[2.62.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.19...v2.62.20\n\n[2.62.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.18...v2.62.19\n\n[2.62.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.17...v2.62.18\n\n[2.62.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.16...v2.62.17\n\n[2.62.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.15...v2.62.16\n\n[2.62.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.14...v2.62.15\n\n[2.62.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.13...v2.62.14\n\n[2.62.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.12...v2.62\n\n> ✂ **Note**\n> \n> PR body was truncated to here.\n\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: (UTC)\n\n- Branch creation\n  - \"before 8am on the first day of the month\"\n- Automerge\n  - At any time (no schedule defined)\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n👻 **Immortal**: This PR will be recreated if closed unmerged. Get\n[config\nhelp](https://redirect.github.com/renovatebot/renovate/discussions) if\nthat's undesired.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My4xNTkuMiIsInVwZGF0ZWRJblZlciI6IjQzLjE1OS4yIiwidGFyZ2V0QnJhbmNoIjoibWFpbiIsImxhYmVscyI6WyJkZXBlbmRlbmNpZXMiXX0=-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2026-05-02T11:20:16Z",
          "tree_id": "b4b323c10b769203f65d4896f59fc3f9dcafc342",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/9aa767ee7b26712bbab69e4ecab5db2b22f80f32"
        },
        "date": 1777734220337,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.9887005090713501,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 5.76603143711191,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 6.236069310743165,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.661067708333334,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.5859375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 6041.2645084442975,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 6100.994524770725,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003332,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 214192.37834134945,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 177281.12712983164,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "lalit_fin@yahoo.com",
            "name": "Lalit Kumar Bhasin",
            "username": "lalitb"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "bca750e798b12053aa0c0c44e7393dfc4f7578e6",
          "message": "Add host metrics receiver design (#2799)\n\n# Change Summary\n\n\nAdds a design doc for the Linux v1 OTAP-native `host_metrics` receiver\nproposed in open-telemetry/otel-arrow#2741. Most of the direction is\nalready established in the issue. This document mainly makes the Rust\nOTAP Dataflow implementation choices explicit before code is added. So\nspecifically covering:\n\n\n  - singleton one-core receiver deployment\n  - per-family interval scheduling\n  - shared Linux scrape snapshots\n  - `HostView` / `root_path` handling\n  - duplicate host collection guard\n  - direct OTAP metrics emission\n  - semantic convention projection\n  - receiver self-observability through `MetricSet`\n  - partial failure behavior\n  - validation and benchmark scope\n\n## What issue does this PR close?\n\n\n* Towards #2741 \n\n## How are these changes tested?\n\n  - `npx --yes markdownlint-cli docs/host-metrics-receiver.md`\n  - `python3 tools/sanitycheck.py`\n\n## Are there any user-facing changes?\n\nNot yet.\n\n---------\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2026-05-03T03:10:18Z",
          "tree_id": "7fbf755a5ba6651876e4826091980e5923cc0492",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/bca750e798b12053aa0c0c44e7393dfc4f7578e6"
        },
        "date": 1777780505609,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -2.578796625137329,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 5.760963573772745,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 6.260943190661479,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.576171875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.8828125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 5955.889658851261,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 6109.47993658668,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003798,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 212952.4821777077,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 176021.97242392425,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
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
          "id": "140f30edbbf4ea059eee3823f63c32d7fcab3786",
          "message": "Update docker digest updates (#2800)\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n|---|---|---|---|\n| alpine | final | digest | `2510918` → `5b10f43` |\n| docker.io/alpine | final | digest | `2510918` → `5b10f43` |\n| docker.io/rust | stage | digest | `4a7e3a0` → `a9cfb75` |\n| golang | stage | digest | `595c784` → `b54cbf5` |\n| python | final | digest | `fb83750` → `5b3879b` |\n\n---\n\n### Configuration\n\n📅 **Schedule**: (UTC)\n\n- Branch creation\n  - \"before 8am on the first day of the month\"\n- Automerge\n  - At any time (no schedule defined)\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n👻 **Immortal**: This PR will be recreated if closed unmerged. Get\n[config\nhelp](https://redirect.github.com/renovatebot/renovate/discussions) if\nthat's undesired.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My4xNTkuMiIsInVwZGF0ZWRJblZlciI6IjQzLjE1OS4yIiwidGFyZ2V0QnJhbmNoIjoibWFpbiIsImxhYmVscyI6WyJkZXBlbmRlbmNpZXMiXX0=-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2026-05-03T03:11:09Z",
          "tree_id": "450e5055d076e18cb3446c6a400d1dfefc144353",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/140f30edbbf4ea059eee3823f63c32d7fcab3786"
        },
        "date": 1777781540335,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.860832154750824,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 5.729241953897027,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 6.2430795393058665,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.715625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.03125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 5946.850820671546,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5998.043223718933,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.008904,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 213305.14952179143,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 176073.20670369727,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
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
          "id": "fe469abf54b3700a301deeab1cd987722df96382",
          "message": "Update github workflow dependencies (major) (#2803)\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n|---|---|---|---|\n|\n[actions/github-script](https://redirect.github.com/actions/github-script)\n| action | major | `v8.0.0` → `v9.0.0` |\n| [dorny/test-reporter](https://redirect.github.com/dorny/test-reporter)\n| action | major | `v2.7.0` → `v3.0.0` |\n\n---\n\n### Release Notes\n\n<details>\n<summary>actions/github-script (actions/github-script)</summary>\n\n###\n[`v9.0.0`](https://redirect.github.com/actions/github-script/releases/tag/v9.0.0)\n\n[Compare\nSource](https://redirect.github.com/actions/github-script/compare/v8.0.0...v9.0.0)\n\n**New features:**\n\n- **`getOctokit` factory function** — Available directly in the script\ncontext. Create additional authenticated Octokit clients with different\ntokens for multi-token workflows, GitHub App tokens, and cross-org\naccess. See [Creating additional clients with\n`getOctokit`](https://redirect.github.com/actions/github-script#creating-additional-clients-with-getoctokit)\nfor details and examples.\n- **Orchestration ID in user-agent** — The `ACTIONS_ORCHESTRATION_ID`\nenvironment variable is automatically appended to the user-agent string\nfor request tracing.\n\n**Breaking changes:**\n\n- **`require('@&#8203;actions/github')` no longer works in scripts.**\nThe upgrade to `@actions/github` v9 (ESM-only) means\n`require('@&#8203;actions/github')` will fail at runtime. If you\npreviously used patterns like `const { getOctokit } =\nrequire('@&#8203;actions/github')` to create secondary clients, use the\nnew injected `getOctokit` function instead — it's available directly in\nthe script context with no imports needed.\n- `getOctokit` is now an injected function parameter. Scripts that\ndeclare `const getOctokit = ...` or `let getOctokit = ...` will get a\n`SyntaxError` because JavaScript does not allow `const`/`let`\nredeclaration of function parameters. Use the injected `getOctokit`\ndirectly, or use `var getOctokit = ...` if you need to redeclare it.\n- If your script accesses other `@actions/github` internals beyond the\nstandard `github`/`octokit` client, you may need to update those\nreferences for v9 compatibility.\n\n##### What's Changed\n\n- Add ACTIONS\\_ORCHESTRATION\\_ID to user-agent string by\n[@&#8203;Copilot](https://redirect.github.com/Copilot) in\n[#&#8203;695](https://redirect.github.com/actions/github-script/pull/695)\n- ci: use deployment: false for integration test environments by\n[@&#8203;salmanmkc](https://redirect.github.com/salmanmkc) in\n[#&#8203;712](https://redirect.github.com/actions/github-script/pull/712)\n- feat!: add getOctokit to script context, upgrade\n[@&#8203;actions/github](https://redirect.github.com/actions/github) v9,\n[@&#8203;octokit/core](https://redirect.github.com/octokit/core) v7, and\nrelated packages by\n[@&#8203;salmanmkc](https://redirect.github.com/salmanmkc) in\n[#&#8203;700](https://redirect.github.com/actions/github-script/pull/700)\n\n##### New Contributors\n\n- [@&#8203;Copilot](https://redirect.github.com/Copilot) made their\nfirst contribution in\n[#&#8203;695](https://redirect.github.com/actions/github-script/pull/695)\n\n**Full Changelog**:\n<https://github.com/actions/github-script/compare/v8.0.0...v9.0.0>\n\n</details>\n\n<details>\n<summary>dorny/test-reporter (dorny/test-reporter)</summary>\n\n###\n[`v3.0.0`](https://redirect.github.com/dorny/test-reporter/releases/tag/v3.0.0)\n\n[Compare\nSource](https://redirect.github.com/dorny/test-reporter/compare/v2.7.0...v3.0.0)\n\n**Note:** The v3 release requires NodeJS 24 runtime on GitHub Actions\nrunners.\n\n#### What's Changed\n\n- Upgrade action runtime to Node.js 24 by\n[@&#8203;dav-tb](https://redirect.github.com/dav-tb) in\n[#&#8203;738](https://redirect.github.com/dorny/test-reporter/pull/738)\n- Explicitly use lowest permissions required to run workflow by\n[@&#8203;jozefizso](https://redirect.github.com/jozefizso) in\n[#&#8203;745](https://redirect.github.com/dorny/test-reporter/pull/745)\n\n##### Other Changes\n\n- Bump\n[@&#8203;typescript-eslint/parser](https://redirect.github.com/typescript-eslint/parser)\nfrom 8.57.0 to 8.57.1 by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;742](https://redirect.github.com/dorny/test-reporter/pull/742)\n- Bump\n[@&#8203;types/adm-zip](https://redirect.github.com/types/adm-zip) from\n0.5.7 to 0.5.8 by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;743](https://redirect.github.com/dorny/test-reporter/pull/743)\n- Bump flatted from 3.4.1 to 3.4.2 by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;744](https://redirect.github.com/dorny/test-reporter/pull/744)\n\n#### New Contributors\n\n- [@&#8203;dav-tb](https://redirect.github.com/dav-tb) made their first\ncontribution in\n[#&#8203;738](https://redirect.github.com/dorny/test-reporter/pull/738)\n\n**Full Changelog**:\n<https://github.com/dorny/test-reporter/compare/v2.7.0...v3.0.0>\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: (UTC)\n\n- Branch creation\n  - \"before 8am on the first day of the month\"\n- Automerge\n  - At any time (no schedule defined)\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n👻 **Immortal**: This PR will be recreated if closed unmerged. Get\n[config\nhelp](https://redirect.github.com/renovatebot/renovate/discussions) if\nthat's undesired.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My4xNTkuMiIsInVwZGF0ZWRJblZlciI6IjQzLjE1OS4yIiwidGFyZ2V0QnJhbmNoIjoibWFpbiIsImxhYmVscyI6WyJkZXBlbmRlbmNpZXMiXX0=-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2026-05-03T03:11:43Z",
          "tree_id": "b2739e400bbc869d5f779c7cb6de470a71baf203",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/fe469abf54b3700a301deeab1cd987722df96382"
        },
        "date": 1777782577113,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.4184396266937256,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 5.762075568083611,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 6.220285119827387,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.979166666666668,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.34375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 6015.193262330301,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 6100.51515257612,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.008047,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 212549.1895025614,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 175588.1405715943,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "AaronRM@users.noreply.github.com",
            "name": "Aaron Marten",
            "username": "AaronRM"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "00b00e301d89f9b298ef3b5d8256d210b497fc2f",
          "message": "fix(tests): increase shutdown deadlines from 200ms to 1s in parquet adaptive_schema tests (#2812)\n\n# Change Summary\n\nIncrease shutdown timeouts in parquet adaptive_schema tests to address\ntest flakiness on slow CI runs.\n\n## What issue does this PR close?\n\n* Addresses a flaky test reported in #2720 \n\n## How are these changes tested?\n\n* Verified tests pass locally\n\n## Are there any user-facing changes?\n\nNo. This is a test-only change.",
          "timestamp": "2026-05-04T16:09:28Z",
          "tree_id": "e000a86f0b4799ab6c345f344796d646e134257b",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/00b00e301d89f9b298ef3b5d8256d210b497fc2f"
        },
        "date": 1777921854288,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.42796003818511963,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 5.813802893372647,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 6.517434826332483,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.577734375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.9375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 5981.311301912284,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 6006.908925172964,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.005571,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 213465.2698569727,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 176525.44291488072,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "AaronRM@users.noreply.github.com",
            "name": "Aaron Marten",
            "username": "AaronRM"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "34c171b819eb69c1fce176e54072729c747b0cd0",
          "message": "fix(tests): increase timeout duration for logs and metrics watch tests (#2817)\n\n# Change Summary\n\nThe `metrics_watch_human_color_always_styles_stream_header` and\n`logs_watch_uses_next_seq_as_after_cursor` tests were using a 10ms\ntimeout that was too tight for HTTP requests to the mock server.\n\nIncrease the timeouts from 10ms to 200ms to provide sufficient time for:\n - Mock server HTTP connection establishment\n - Request/response round-trip\n - Output rendering and writing to stdout\n\n## What issue does this PR close?\n\n* Addresses flaky test\n`metrics_watch_human_color_always_styles_stream_header` from #2720\n\n## How are these changes tested?\n\nValidated that tests pass locally\n\n## Are there any user-facing changes?\n\nNo. This is a test only change.",
          "timestamp": "2026-05-04T16:35:35Z",
          "tree_id": "a745055661847578c3c0253e7f8d41887ab3f8bc",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/34c171b819eb69c1fce176e54072729c747b0cd0"
        },
        "date": 1777922905488,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.4164305925369263,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 5.753190380968878,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 6.137920792079208,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.070052083333334,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.19921875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 6024.1824247070945,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 6109.510787663286,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003495,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 213513.90237574186,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 176579.02801431302,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
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
          "id": "b91157891d6b1a4b3d3198d24bb42c6d0209be02",
          "message": "Add try_into_with_options with empty ConversionOptions struct (#2811)\n\n# Change Summary\n\nReplaces TryInto by a new TryIntoWithOptions and a default path, then\nmechanically converts the code.\n\n## What issue does this PR close?\n\nPart of #2725 \nSplit from #2792 \n\n## Are there any user-facing changes?\n\nNone.\n\n---------\n\nCo-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>",
          "timestamp": "2026-05-04T19:29:08Z",
          "tree_id": "f765582949669a202a6dc8062f3577e5ebffcb02",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/b91157891d6b1a4b3d3198d24bb42c6d0209be02"
        },
        "date": 1777926859299,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.1363636255264282,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 5.743218680874361,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 6.254475659778655,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.274869791666667,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.69921875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 6006.40053057249,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5938.145979088712,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.01065,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 213042.1333061851,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 176116.32083185203,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
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
          "id": "f5f798f0fc2c2e0760b0879f452431172680393f",
          "message": "Implement pdata-propagated stopwatch timer across processors (#2747)\n\n# Change Summary\n\nAlternative to the full `ProcessorChain` implementation under discussion\nat #2556\n\nAfter some conversations, trying to simplify back to focusing on ONLY\nthe composite duration problem in a way that isn't directly tied to the\nchannel elimination strategy / full implementation of `ProcessorChain`\n\n## Motivation\n\nPer-node `processor.compute.*.duration` reports each processor in\nisolation. Operators also need a single metric for the combined compute\nof a *range* of processors (e.g. five sequential attribute processors as\none number).\n\nDistributed **stopwatches** are engine-managed metrics that sum\nper-message compute time across configurable processor node pairs on the\nforward path.\n\n## Design\n\n**What's measured.** The wall-clock time spent inside each processor's\n`process()` between successive sends. Queue wait and channel latency\nbetween nodes are excluded.\n\n**How it's measured.** Immediately before each `process()` call, the\nengine arms an `Instant` send-marker on the EffectHandler. On every send\n(`send_message`, `try_send_message`, `_to`, and `_with_source_node*`\nvariants), the unified `ProcessorSendHook` reads the elapsed time since\nthe marker, advances the marker to \"now\", and adds the delta to the\nmessage's on-PData `stopwatch_compute_ns` accumulator. No processor\ncooperation needed — `timed()` is not required for stopwatch\nparticipation, and is used solely to split per-node compute into\nsuccess/failed.\n\n**Non-overlapping ranges.** At most one stopwatch is active per message.\nThe on-PData accumulator is a single `Option<NonZeroU64>`.\n\n**Declarative YAML config.**\n\n```yaml\npolicies:\n  telemetry:\n    runtime_metrics: normal\n    stopwatches:\n      - name: ingest_pipeline\n        start_node: attr1\n        stop_node: attr5\n```\n\nEach stopwatch gets its own metric entity with `stopwatch.name`,\n`stopwatch.start_node`, and `stopwatch.stop_node` attributes.\n\n## How It Works\n\n1. **Build time** — `build_stopwatch_state()` resolves config node names\nto indices, validates that endpoints are processor nodes (local or\nshared; receivers and exporters are rejected) and that no node is reused\nacross roles. If `runtime_metrics` does not include `PROCESS_DURATION`,\nall stopwatches are skipped with a single warning. Each processor's\n`EffectHandler` is assigned its start/stop role and (for stop nodes) the\nmetric set to report into.\n2. **Marker arming** — the engine loop calls\n`effect_handler.begin_process_timing()` before each `process()` call\nwhen stopwatches are active.\n   - Local: `Rc<Cell<Option<Instant>>>`.\n- Shared: `Arc<Mutex<Option<Instant>>>` (mutex contention is bounded to\nthe per-processor sequential `process()` loop).\n3. **Forward-path accumulation** — `OtapPdata::before_processor_send`\n(the unified `ProcessorSendHook` impl, called by both the plain\n`send_message*` family and the `send_message_with_source_node*` family\non local and shared processor handlers) invokes `stopwatch_accumulate`,\nwhich calls `take_elapsed_since_send_marker_ns()` (single\nread-and-advance step) and:\n- **Start node:** initialises the on-PData accumulator with the 1 ns\nsentinel.\n   - **Any node within an active range:** adds the delta.\n- **Stop node:** takes the total via `take_stopwatch_compute()` and\nrecords into the local stop accumulator (`Cell<Mmsc>` for local,\n`Arc<Mutex<Mmsc>>` for shared).\n4. **Periodic reporting** — on `CollectTelemetry` and at shutdown, the\nengine loop calls `effect_handler.report_stopwatch()`, which drains the\nstop accumulator into its `MetricSet` and reports it.\n\n## Metrics Produced\n\n| Metric Set | Metric Name | Unit | Attributes |\n\n|-------------|------------------------------|------|---------------------------------------------------------------------------------|\n| `stopwatch` | `stopwatch.compute.duration` | ns | `stopwatch.name`,\n`stopwatch.start_node`, `stopwatch.stop_node`, pipeline attrs |\n\n`Mmsc` (min/max/sum/count), delta temporality.\n\n## Live Validation\n\nDemo: `configs/fake-stopwatch-demo.yaml` (5 attribute processors, one\nstopwatch spanning the chain).\n\n```bash\ncurl -s 'http://127.0.0.1:8080/api/v1/telemetry/metrics?format=json&reset=true' | jq '\n  [.metric_sets[] | select(.name == \"stopwatch\" or .name == \"processor.compute\")]\n  | .[] | {\n      name: .name,\n      identity: (.attributes[\"node.id\"].String // .attributes[\"stopwatch.name\"].String),\n      duration: (\n        .metrics[]\n        | select(\n            (.name == \"compute.success.duration\") or\n            (.name == \"stopwatch.compute.duration\")\n          )\n        | .value\n        | if type == \"object\" and .count > 0 then {\n            avg_ms: (.sum / .count / 1e6),\n            count\n          } else empty end\n      )\n    }'\n```\n\nThe stopwatch average is at least the sum of per-processor\n`processor.compute` averages — it also covers `process()` time outside\n`timed()` closures (message handling, send-side bookkeeping):\n\n| Metric | Node / Stopwatch | Avg (ms) | Count |\n\n|---------------------|-----------------------------------|-----------|--------|\n| `processor.compute` | attr1 | 1.305 | 40 |\n| `processor.compute` | attr2 | 0.908 | 40 |\n| `processor.compute` | attr3 | 1.207 | 40 |\n| `processor.compute` | attr4 | 0.521 | 40 |\n| `processor.compute` | attr5 | 0.519 | 40 |\n| **`stopwatch`** | **ingest_pipeline** (attr1→attr5) | **6.121** |\n**40** |\n\n## Limitations / Follow-ups\n\n- **Non-overlapping only.** Build-time validation rejects nodes reused\nacross roles (e.g. `a→b` + `b→c`); runtime detection warns and resets if\ntopologically overlapping ranges (e.g. 1→3 + 2→4) appear.\n- **Requires `runtime_metrics: normal` or higher.** Gated behind the\nsame `PROCESS_DURATION` interest as `processor.compute`. Lower levels\nskip all stopwatches at build time with one warning.\n- **Processors only.** Receivers and exporters are rejected at build\ntime (no incoming-message frame / no further send path).\n- **No topology ordering validation.** Start-before-stop is not\nenforced; a misconfigured stopwatch simply never fires.\n- **Shared processors with worker tasks.** The marker captures only time\nspent inside `process()` itself. Processors that fan work out to spawned\ntasks and return early can still report accurate per-message compute via\n`timed()` from inside their workers.\n- **Send-bounded measurement.** Each processor's contribution is the\nwall-clock span from `begin_process_timing` (or its previous send)\nthrough its next `send_message`. Two consequences:\n- **Dropped messages contribute nothing.** A processor that consumes a\nmessage without sending it never reaches a stop node, so no stopwatch\nobservation is recorded — by design, the metric counts only complete\ntransits.\n- **Post-final-send work is not attributed.** Cleanup performed after\nthe last `send_message` in `process()` is not captured. For typical\nstateless processors this is a trivial return path; processors that do\nsignificant post-send work should perform it before the final send if\nthey want it measured.\n\n## What issue does this PR close?\n\n* Related to #2556\n* Contributes to #2782\n\n## How are these changes tested?\n\nUnit tests, local run of sample config\n\n## Are there any user-facing changes?\n\nYes, users could configure stopwatch metrics across multiple processors",
          "timestamp": "2026-05-04T23:17:49Z",
          "tree_id": "cf6955d2d8ba7477dddc57643ae666943bba9521",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/f5f798f0fc2c2e0760b0879f452431172680393f"
        },
        "date": 1777940295737,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -2.435530185699463,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 5.789262284676699,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 6.3603939511438545,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.002734375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.578125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 5954.5924337607075,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 6099.618323981241,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.01687,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 214579.3973644053,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 177697.69492671313,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "33842784+JakeDern@users.noreply.github.com",
            "name": "Jake Dern",
            "username": "JakeDern"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "1884c67d731ae445dcab2d6642d7344b37bbad38",
          "message": "fix(otlp_grpc_exporter): Set accept_compressed in addition to send_compressed (#2829)\n\n# Change Summary\n\nSet `accept_compressed` so that we can process responses if the replies\nare compressed with the same codec.\n\n## What issue does this PR close?\n\n\n* Closes #2828\n\n## How are these changes tested?\n\nI tried them locally and verified the warnings in the issue went away.\n\n## Are there any user-facing changes?\n\nNo.",
          "timestamp": "2026-05-04T23:33:25Z",
          "tree_id": "5320b9c5a6c8d4ab00dc7b87d62023ae0e477125",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/1884c67d731ae445dcab2d6642d7344b37bbad38"
        },
        "date": 1777941342653,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.9900990128517151,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 5.78102318177053,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 6.248904713987151,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.01328125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.76171875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 6032.745422972894,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 6092.475575675596,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003195,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 213528.98204685643,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 176473.58791924306,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "33842784+JakeDern@users.noreply.github.com",
            "name": "Jake Dern",
            "username": "JakeDern"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "11fdb97ee8b5899cc280f6a64a812a9f024987a8",
          "message": "feat(orchestrator): Allow providing a `--tests` flag to filter the tests to be run (#2830)\n\n# Change Summary\n\nWhen working on benchmarks I often need to run or re-run just a single\ntest of a much larger suite. Rather than play around with commenting\nstuff in and out, especially when templates are involved, it's much\neasier to pass this argument.\n\n## What issue does this PR close?\n\nNone :(\n\n## How are these changes tested?\n\nI've been running this daily locally.\n\n## Are there any user-facing changes?\n\nJust for the orchestrator - New `--tests` flag.",
          "timestamp": "2026-05-05T01:48:31Z",
          "tree_id": "09deb0da8cf0950d8a85e84475ad48841f470366",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/11fdb97ee8b5899cc280f6a64a812a9f024987a8"
        },
        "date": 1777948949283,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -2,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 5.755670186209963,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 6.260938271604938,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.992578125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.37109375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 5972.989189606193,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 6092.448973398316,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003457,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 214398.53746347738,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 176843.54187603868,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "lalit_fin@yahoo.com",
            "name": "Lalit Kumar Bhasin",
            "username": "lalitb"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "8b1e57dbc1f5a79d100ed8f3e1c2aeb42fe89417",
          "message": "  [Geneva Exporter] Encode logs using LogsDataView (#2810)\n\n## Description\n\nUpdates the otel-arrow Geneva exporter to encode log payloads through\nthe shared `LogsDataView` abstraction instead of converting logs back\nthrough OTLP proto objects.\n\n  ## Changes\n\n- Encode OTAP Arrow logs through `OtapLogsView`, the OTAP implementation\nof `LogsDataView`.\n- Encode OTLP byte logs through `RawLogsData`, the OTLP bytes\nimplementation of `LogsDataView`.\n- Call the existing `geneva-uploader` `encode_and_compress_logs(&view)`\nAPI for both log paths.\n- Patch `otap-df-pdata` / `otap-df-pdata-views` so `geneva-uploader`\nuses the workspace view traits.\n- Add a TODO for compressed byte accounting until\n`EncodedBatch::data_len()` is exposed upstream.\n\n## How are these changes tested?\n\n  - `cargo fmt --all`\n- `CARGO_NET_GIT_FETCH_WITH_CLI=true cargo check -p\notap-df-contrib-nodes --features \"geneva-exporter\notap-df-otap/crypto-ring\"`\n\n## Are there any user-facing changes?\n\n <!-- If yes, provide further info below -->\nNone.",
          "timestamp": "2026-05-05T03:06:26Z",
          "tree_id": "15db94aa98fd779b4698b4f3b6525799f4b6f2bc",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/8b1e57dbc1f5a79d100ed8f3e1c2aeb42fe89417"
        },
        "date": 1777958279151,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.4285714030265808,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 5.750804913493768,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 6.522168767886146,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.658984375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.73046875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 5972.687586260466,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5998.284818773011,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.006487,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 212755.42665633943,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 175713.9953720253,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
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
          "id": "8b4e674f6909f04cdd6726c42e3ad0f3281b3d6e",
          "message": "Update module go.opentelemetry.io/collector/pdata to v1.57.0 (#2815)\n\nThis PR contains the following updates:\n\n| Package | Change |\n[Age](https://docs.renovatebot.com/merge-confidence/) |\n[Confidence](https://docs.renovatebot.com/merge-confidence/) |\n|---|---|---|---|\n|\n[go.opentelemetry.io/collector/pdata](https://redirect.github.com/open-telemetry/opentelemetry-collector)\n| `v1.56.0` → `v1.57.0` |\n![age](https://developer.mend.io/api/mc/badges/age/go/go.opentelemetry.io%2fcollector%2fpdata/v1.57.0?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/go/go.opentelemetry.io%2fcollector%2fpdata/v1.56.0/v1.57.0?slim=true)\n|\n\n---\n\n### Release Notes\n\n<details>\n<summary>open-telemetry/opentelemetry-collector\n(go.opentelemetry.io/collector/pdata)</summary>\n\n###\n[`v1.57.0`](https://redirect.github.com/open-telemetry/opentelemetry-collector/blob/HEAD/CHANGELOG.md#v1570v01510)\n\n##### 🛑 Breaking changes 🛑\n\n- `cmd/builder`: In the generated Collector source, the `replace`\nstatements in the Go module will now use relative paths by default.\n([#&#8203;15097](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/15097))\nWe expect that this will not break existing use-cases where the\ngenerated collector is only used in an interim manner for builds. It\nenables the possibility of tracking the generated Collector code as a\nlonger living artifact, allowing it to be run on any machine (whereas\nabsolute paths will be different depending on the machine the Collector\nsource is generated on.) We have added\n`dist::use_absolute_replace_paths` to go back to the absolute path\nbehaviour in the case where there is an unforeseen use-case that\nrequires absolute paths.\n\n- `pkg/confighttp`: Stabilize framedSnappy feature gate.\n([#&#8203;15096](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/15096))\n\n##### 💡 Enhancements 💡\n\n- `all`: Add declarative schema support for service telemetry resource\nconfiguration.\n([#&#8203;14411](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/14411))\nThe `service::telemetry::resource` configuration now accepts the\ndeclarative schema with explicit name/value pairs:\n\n  ```yaml\n  service:\n    telemetry:\n      resource:\n        schema_url: https://opentelemetry.io/schemas/1.38.0\n        attributes:\n          - name: service.name\n            value: my-collector\n          - name: host.name\n            value: collector-host\n  ```\n\nThe legacy inline attribute map format is still supported for backward\ncompatibility:\n\n  ```yaml\n  service:\n    telemetry:\n      resource:\n        service.name: my-collector\n        host.name: collector-host\n  ```\n\nNote: `resource.detectors` is accepted for forward compatibility but is\nnot yet applied by the collector.\n\n- `exporter/otlp_grpc`: Added the `server.address` and `url.path`\nattributes to metrics generated by the otlp exporter.\n([#&#8203;14998](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/14998))\n\n- `exporter/otlp_http`: Added the `server.address` and `url.path`\nattributes to metrics generated by the otlp\\_http exporter.\n([#&#8203;14998](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/14998))\n\n- `pkg/config/configgrpc`: Add `UserAgent` field to `ClientConfig` to\nallow overriding the default gRPC user-agent string.\n([#&#8203;14686](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/14686))\n  The otlp gRPC exporter was unconditionally setting the User-Agent via\ngrpc.WithUserAgent() at dial time, which takes precedence over per-call\nmetadata, causing any user-configured User-Agent to be silently\ndiscarded.\nA dedicated `UserAgent` field has been added to `ClientConfig` which,\nwhen\nset, is used in the dial option directly instead of the default\nBuildInfo-derived string.\n\n- `pkg/config/configgrpc`: Accept gRPC resolver scheme URIs in client\nendpoint (e.g. passthrough:///host:port) to allow control over name\nresolution\n([#&#8203;14990](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/14990))\nAfter the migration to grpc.NewClient, some gRPC client components such\nas the OTLP\nexporter experienced connection issues in dual-stack DNS environments.\nThis can now be\nfixed by using the passthrough:/// gRPC resolver scheme in the endpoint\nfield.\n\n- `pkg/config/confignet`: Add support for Windows Named Pipe (npipe)\ntransport\n([#&#8203;15085](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/15085))\n\n- `pkg/service`: Emit a warning when using the old v0.2.0 declarative\nconfig format\n([#&#8203;15088](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/15088))\n\n##### 🧰 Bug fixes 🧰\n\n- `pkg/otelcol`: Print components exactly once in the `otelcol\ncomponents` command\n([#&#8203;14682](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/14682))\n  This resolves an issue where aliased components were skipped.\n\n- `pkg/otelcol`: Synchronize Collector Run and Shutdown lifecycles so\nthat Shutdown blocks until Run completes all cleanup.\n([#&#8203;4947](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/4947))\nShutdown now blocks until Run finishes cleanup, matching http.Server\nsemantics.\nIf Shutdown is called before Run, the next Run call returns nil after\ncleaning up\n  the config provider.\n\n- `pkg/pdata`: Use spec-compliant string representation for NaN,\nInfinity, and -Infinity in Value.AsString().\n([#&#8203;14487](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/14487))\n\n- `pkg/pprofile`: Fix data corruption of resource and scope attributes\nafter marshal-unmarshal-merge round-trip.\n([#&#8203;15084](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/15084))\n\n- `pkg/service`: Non-string resource attributes in telemetry\nconfiguration now return an error instead of panicking\n([#&#8203;15171](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/15171))\n\n- `pkg/xscraperhelper`: fix the merge of profiles in the profiling\nscraper helpers\n([#&#8203;14790](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/14790))\n\n- `receiver/otlp`: Fix profiles receiver reporting its samples as spans\n([#&#8203;15089](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/15089))\n\n<!-- previous-version -->\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: (UTC)\n\n- Branch creation\n  - \"before 8am on Monday\"\n- Automerge\n  - At any time (no schedule defined)\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My4xNTkuMiIsInVwZGF0ZWRJblZlciI6IjQzLjE1OS4yIiwidGFyZ2V0QnJhbmNoIjoibWFpbiIsImxhYmVscyI6WyJkZXBlbmRlbmNpZXMiXX0=-->\n\n---------\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: otelbot <197425009+otelbot@users.noreply.github.com>\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-05-05T12:14:50Z",
          "tree_id": "96351c4e7feacba2bb2ed0e4535202693efbe663",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/8b4e674f6909f04cdd6726c42e3ad0f3281b3d6e"
        },
        "date": 1777988777639,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.1299434900283813,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 5.814225004285036,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 6.463173203372785,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.235677083333332,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.79296875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 6041.245781622338,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 6109.50844582146,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003518,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 213831.81655335153,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 176905.14712334104,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
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
          "id": "f0f16046495687c7e36810683ff10e0fde7b1eaa",
          "message": "Update geneva-uploader digest to ce866b4 (#2831)\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n|---|---|---|---|\n| geneva-uploader | workspace.dependencies | digest | `0022519` →\n`ce866b4` |\n\n---\n\n### Configuration\n\n📅 **Schedule**: (UTC)\n\n- Branch creation\n  - At any time (no schedule defined)\n- Automerge\n  - At any time (no schedule defined)\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My4xNTkuMiIsInVwZGF0ZWRJblZlciI6IjQzLjE1OS4yIiwidGFyZ2V0QnJhbmNoIjoibWFpbiIsImxhYmVscyI6WyJkZXBlbmRlbmNpZXMiXX0=-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: Lalit Kumar Bhasin <lalit_fin@yahoo.com>",
          "timestamp": "2026-05-05T16:57:34Z",
          "tree_id": "718c7b68ba983548ed2496f6d9c8ce68080be9ba",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/f0f16046495687c7e36810683ff10e0fde7b1eaa"
        },
        "date": 1778014467176,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.133144497871399,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 5.682583046823742,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 6.0351350932879155,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.691666666666666,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.06640625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 6024.195074779885,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 6092.457908488438,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003369,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 212632.376024336,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 175635.90974702814,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
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
          "id": "29de46bb4dbff6e48b595459188f912b49373eed",
          "message": "Add capability to evaluate field-level type checks (#2794)\n\n# Change Summary\n\n<!--\nReplace with a brief summary of the change in this PR\n-->\n\nPart 2 of what was started in\nhttps://github.com/open-telemetry/otel-arrow/pull/2754 ...\n\nAdds the capability to OPL and the OTAP query-engine to evaluate logical\nexpressions that check the type of some field. For example:\n```kql\nlogs | where attributes[\"x\"] is String\nlogs | where attributes[\"y\"] is Integer\n```\n\nThis i useful for cases where users may wish to do something with a\nfield of unknown type (an `AnyValue`) and the operation would fail if\nthe value was not the correct type. For example, maybe I wish to redact\nlog body only if it is text, when I know that sometimes the body will\nactually be an int:\n```kotlin\nlogs | if (body is String) {\n  set body = concat(substring(body, 0, 3), \"****\") // <-- substring only accepts string type\n}\n```\n\nIn this case we use the `GetType` expression which is only supposed to\nreturn a value from the `ValueType` enum (see discussion here:\nhttps://github.com/open-telemetry/otel-arrow/pull/2754#discussion_r3139794049),\nso currently only these values are supported. Although there are in fact\nmany other types supported by OTel (e.g. specific int types like uint32,\nsome etc.) the set of types in `ValueType` seem to be enough for most\nuseful expressions, at least in the context of what the engine currently\nsupports.\n\n## What issue does this PR close?\n\n<!--\nWe highly recommend correlation of every PR to an issue\n-->\n\n* Closes #2752\n\n## How are these changes tested?\n\nUnit tests\n\n## Are there any user-facing changes?\n\nYes - this type of expressions is now available in transform processor.\n\n <!-- If yes, provide further info below -->\n\n---------\n\nCo-authored-by: Joshua MacDonald <jmacd@users.noreply.github.com>",
          "timestamp": "2026-05-05T18:19:33Z",
          "tree_id": "95bf22e723ad09cb009d8b25f8b0c193629d3c0b",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/29de46bb4dbff6e48b595459188f912b49373eed"
        },
        "date": 1778020367685,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.2729843854904175,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 5.766881139687753,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 6.425954647473106,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.635286458333333,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.203125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 6032.722801466984,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 6109.5184241164925,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00342,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 213622.2294589405,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 176760.9256996319,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "d.dahl@f5.com",
            "name": "David Dahl",
            "username": "daviddahl"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "21b02e791e3af690fcb61d44e27b9311f597b85a",
          "message": "[PerfTest] Fix shutdown hooks: use wait=false with readyz polling (#2795) (#2796)\n\n## Summary\n\nFixes #2795.\n\nThe `df-loadgen-steps-docker*.yaml` test step templates used `wait=true`\non shutdown hooks, which blocks until all pipelines terminate. The admin\nserver **stays alive after pipeline shutdown by design** (`run_forever`\nparks the main thread), so `wait=true` returns **504 Gateway Timeout**\nwhen drain takes longer than `timeout_secs`.\n\nSince `send_http_request` defaults to `raise_for_status: true` with no\n`on_error` configured, this 504 aborts the entire suite before the\nreport step -- losing all metrics collected during the observation\nwindow.\n\n## Changes\n\nSwitch from blocking `wait=true` to non-blocking `wait=false` with\nactive readyz polling across all three affected template files:\n\n| File | Hooks fixed |\n|---|---|\n| `df-loadgen-steps-docker.yaml` | 3 (ports 8085, 8086, 8087) |\n| `df-loadgen-steps-docker-filtered.yaml` | 3 (ports 8085, 8086, 8087) |\n| `df-loadgen-steps-docker-otel.yaml` | 1 (port 8085) |\n\n**Pattern for each shutdown hook:**\n1. `POST .../shutdown?wait=false` -- returns 202 immediately, drain runs\nin background\n2. Poll `/readyz` every 5s until 503 (pipelines terminated) or endpoint\nstops responding\n3. `on_error: continue: true` so 404/connection errors (service already\nexited) are non-fatal\n4. Polling loop exits with a warning after 300s (container will be\nkilled by `destroy` regardless)\n\n## Testing\n\nVerified in downstream deployments using the orchestrator framework\nacross a range of workloads. The polling approach provides clear log\noutput (\"Engine pipelines stopped after Ns\") and never aborts the suite\ndue to shutdown timing.\n\n/cc @cijothomas\n\n---------\n\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-05-05T20:00:56Z",
          "tree_id": "e3c117b03d4a3966f95e81030e882cc3b33aa25e",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/21b02e791e3af690fcb61d44e27b9311f597b85a"
        },
        "date": 1778021613467,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.4285714030265808,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 5.787649060136712,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 6.308656254832225,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.743880208333334,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.0703125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 5972.982420616122,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5998.580916704476,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003525,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 213980.78378728038,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 177652.54382963944,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "AaronRM@users.noreply.github.com",
            "name": "Aaron Marten",
            "username": "AaronRM"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "93af768ae44406440fd200341f8e7fb63784f132",
          "message": "fix(tests): extend backdating duration to prevent flaky tests due to timing jitter (#2842)\n\n# Change Summary\n\nExtends the backdating durations used in the Quiver\n`startup_preserves_fresh_segments_deletes_expired` test to prevent flaky\ntest failures.\n\n## What issue does this PR close?\n\n* Addresses the flaky test result for\n`startup_preserves_fresh_segments_deletes_expired` reported in #2720.\n\n## How are these changes tested?\n\nValidated that test passes locally on re-run.\n\n## Are there any user-facing changes?\n\nNo. This is a test only change.",
          "timestamp": "2026-05-05T20:05:37Z",
          "tree_id": "829dfa521597283206d12e4943acdb5485bb44e7",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/93af768ae44406440fd200341f8e7fb63784f132"
        },
        "date": 1778022651153,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.845070481300354,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 5.76439902938709,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 6.097792900781069,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.417057291666666,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.35546875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 6057.7412457290275,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 6006.549066187656,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.009166,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 212650.40268972504,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 175491.78054221734,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
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
          "id": "a43d415c137bd8d7d82ac55cadcf556cae36ea43",
          "message": "ci(rust): speed up Linux ARM build_nonrequired (#2836)\n\nThe ubuntu-24.04-arm build_nonrequired job has been running ~55 minutes\n(vs ~13 minutes on macos-latest with the same default features), tying\nup the merge queue. Apply the same pattern used to fix the Windows build\nin #2695:\n\n* Use a reduced feature set ('crypto-ring,jemalloc,azure,aws,\nazure-monitor-exporter,contrib-processors,unsafe-optimizations'),\nmirroring the Windows 'Incomplete' variant.\n\nPart of #2591\nFixes #2737\n\n---------\n\nCo-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>",
          "timestamp": "2026-05-05T20:15:52Z",
          "tree_id": "38927bb1bb2d46279c888f66e5bf30ad4a5fdb01",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/a43d415c137bd8d7d82ac55cadcf556cae36ea43"
        },
        "date": 1778023706239,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.5625,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 5.773123608858828,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 6.241242621932277,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.662109375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.07421875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 6007.118353920779,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 6100.9795782007905,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003479,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 213401.48837835185,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 176297.43742972155,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
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
          "id": "3b7ca8b02321fd1e267c17eaab46027a7b474b9e",
          "message": "ci(codecov): wait for all uploads; use auto project target; raise patch to 80% (#2843)\n\nCodecov was reporting a failing project status (e.g. 56% vs target 85%)\nbefore all test jobs had even finished, because it computed the status\nfrom the first upload that arrived. We upload coverage from three jobs:\nrust-ci 'test' (Linux x86_64), rust-ci 'coverage', and go-ci\n'test_and_coverage'. Set codecov.notify.after_n_builds: 3 (with\nwait_for_ci: true) so the status is computed only after all three\nreports have been received.\n\nAlso relax the thresholds:\n- project: switch to 'target: auto' with a 1% threshold so PRs only fail\nif they meaningfully decrease overall coverage, rather than needing the\nwhole repo to hit a fixed 85%.\n- patch: raise to 80% (from 70%) for changed lines in a PR.\n\nFixes #2591\n\n---------\n\nCo-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>",
          "timestamp": "2026-05-05T20:15:58Z",
          "tree_id": "ac11a002a5f77911a9527222697441a1b4741898",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/3b7ca8b02321fd1e267c17eaab46027a7b474b9e"
        },
        "date": 1778024755734,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.4184396266937256,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 5.753230316809878,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 6.297904408925952,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.905338541666666,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.19140625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 6015.635251982555,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 6100.963411585144,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003638,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 213264.26097467577,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 176385.6063760156,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
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
          "id": "5e496a9aaf2b81c147e775b770a542ca6510b8c1",
          "message": "fix(perf-test): flush loadgen metrics during run, not only on thread exit (#2822)\n\nThe Python load generator's three worker threads only flushed\n`logs_produced` / `failed` / `bytes_sent` to the shared metrics dict on\nthread exit, so every Prometheus scrape during the observation window\nsaw `0` — causing the [nightly syslog\ndashboard](https://open-telemetry.github.io/otel-arrow/benchmarks/nightly/syslog/)\nto publish `logs_produced_rate=0` and `dropped_logs_percentage=0` for\nevery syslog test since #2723 (2026-04-23). This PR flushes accumulators\nonce per send-loop iteration; verified locally that `logs_produced_rate`\nis now `~12k` (UDP) / `~95k–200k` (TCP) instead of `0`.\n\nCurrent state where logs count shows 0.\n<img width=\"457\" height=\"667\" alt=\"image\"\nsrc=\"https://github.com/user-attachments/assets/c4d1bfbd-bddc-42c3-919a-d8ee056e64a5\"\n/>",
          "timestamp": "2026-05-05T21:48:25Z",
          "tree_id": "e8921a6d1af4161bbfd118121b6b8c4a1f15f817",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/5e496a9aaf2b81c147e775b770a542ca6510b8c1"
        },
        "date": 1778025789337,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.14064697921276093,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 5.706393195542407,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 6.230330418633445,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.501171875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.69921875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 6066.5087213312045,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 6075.041082401994,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.006837,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 212288.54085377904,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 175445.69880153637,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
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
          "id": "43b48f2f0173874c81756d913b4cef9cd9a58c25",
          "message": "Update all patch versions (#2801)\n\nThis PR contains the following updates:\n\n| Package | Change |\n[Age](https://docs.renovatebot.com/merge-confidence/) |\n[Confidence](https://docs.renovatebot.com/merge-confidence/) |\n|---|---|---|---|\n|\n[github.com/fxamacker/cbor/v2](https://redirect.github.com/fxamacker/cbor)\n| `v2.9.1` → `v2.9.2` |\n![age](https://developer.mend.io/api/mc/badges/age/go/github.com%2ffxamacker%2fcbor%2fv2/v2.9.2?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/go/github.com%2ffxamacker%2fcbor%2fv2/v2.9.1/v2.9.2?slim=true)\n|\n|\n[github.com/klauspost/compress](https://redirect.github.com/klauspost/compress)\n| `v1.18.5` → `v1.18.6` |\n![age](https://developer.mend.io/api/mc/badges/age/go/github.com%2fklauspost%2fcompress/v1.18.6?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/go/github.com%2fklauspost%2fcompress/v1.18.5/v1.18.6?slim=true)\n|\n\n---\n\n### Release Notes\n\n<details>\n<summary>fxamacker/cbor (github.com/fxamacker/cbor/v2)</summary>\n\n###\n[`v2.9.2`](https://redirect.github.com/fxamacker/cbor/releases/tag/v2.9.2)\n\n[Compare\nSource](https://redirect.github.com/fxamacker/cbor/compare/v2.9.1...v2.9.2)\n\nThis release refactors and hardens the streaming encoder by adding\nstricter checks for encoding CBOR indefinite-length data. Other changes\ninclude minor bugfixes, defensive checks, and more tests.\n\nProjects that don't use CBOR indefinite-length data may also want to\nupgrade ([summary of prior\nreleases](https://redirect.github.com/fxamacker/cbor#prior-releases)).\n\nThe stricter checks in the encoder prevent improper use of the library\nand bad inputs from producing malformed CBOR indefinite-length data that\nwould be rejected by the decoder.\n\nThis release passed fuzz tests (billions of execs) and it is production\nquality.\n\n#### What's Changed\n\n- Reject encoding indefinite-length map with odd item count by\n[@&#8203;fxamacker](https://redirect.github.com/fxamacker) in\n[#&#8203;764](https://redirect.github.com/fxamacker/cbor/pull/764)\n- Reject encoding indefinite-length data item as a chunk inside\nindefinite-length byte string or text string by\n[@&#8203;fxamacker](https://redirect.github.com/fxamacker) in\n[#&#8203;765](https://redirect.github.com/fxamacker/cbor/pull/765)\n- Make TagSet.Remove a no-op when contentType is nil by\n[@&#8203;fxamacker](https://redirect.github.com/fxamacker) in\n[#&#8203;766](https://redirect.github.com/fxamacker/cbor/pull/766)\n- Refactor indefinite-length encoding and improve chunk validation\nduring encoding by\n[@&#8203;fxamacker](https://redirect.github.com/fxamacker) in\n[#&#8203;767](https://redirect.github.com/fxamacker/cbor/pull/767)\n- Add more tests, fix a nit in unreachable panic message, update docs &\nci by [@&#8203;fxamacker](https://redirect.github.com/fxamacker) in\n[#&#8203;768](https://redirect.github.com/fxamacker/cbor/pull/768)\n\n##### CI / GitHub Actions and Docs\n\n<details><summary>:mag_right: Details...</summary><p>\n\n- Bump actions/setup-go from 6.3.0 to 6.4.0 by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;760](https://redirect.github.com/fxamacker/cbor/pull/760)\n- Bump github/codeql-action from 4.34.1 to 4.35.1 by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;761](https://redirect.github.com/fxamacker/cbor/pull/761)\n- Bump github/codeql-action from 4.35.1 to 4.35.2 by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;763](https://redirect.github.com/fxamacker/cbor/pull/763)\n- Update README for v2.9.2 release by\n[@&#8203;fxamacker](https://redirect.github.com/fxamacker) in\n[#&#8203;769](https://redirect.github.com/fxamacker/cbor/pull/769)\n\n</details>\n\n**Full Changelog**:\n<https://github.com/fxamacker/cbor/compare/v2.9.1...v2.9.2>\n\n</details>\n\n<details>\n<summary>klauspost/compress (github.com/klauspost/compress)</summary>\n\n###\n[`v1.18.6`](https://redirect.github.com/klauspost/compress/releases/tag/v1.18.6)\n\n[Compare\nSource](https://redirect.github.com/klauspost/compress/compare/v1.18.5...v1.18.6)\n\n#### What's Changed\n\n- s2: Fix amd64 stack frame corruption by\n[@&#8203;klauspost](https://redirect.github.com/klauspost) in\n[#&#8203;1145](https://redirect.github.com/klauspost/compress/pull/1145)\n- gzhttp: Canonicalize ETag header by\n[@&#8203;justinmayhew](https://redirect.github.com/justinmayhew) in\n[#&#8203;1139](https://redirect.github.com/klauspost/compress/pull/1139)\n- perf: pool hash tables in Go encode paths to reduce allocations by\n[@&#8203;huynhanx03](https://redirect.github.com/huynhanx03) in\n[#&#8203;1143](https://redirect.github.com/klauspost/compress/pull/1143)\n\n#### New Contributors\n\n- [@&#8203;justinmayhew](https://redirect.github.com/justinmayhew) made\ntheir first contribution in\n[#&#8203;1139](https://redirect.github.com/klauspost/compress/pull/1139)\n- [@&#8203;huynhanx03](https://redirect.github.com/huynhanx03) made\ntheir first contribution in\n[#&#8203;1143](https://redirect.github.com/klauspost/compress/pull/1143)\n- [@&#8203;thaJeztah](https://redirect.github.com/thaJeztah) made their\nfirst contribution in\n[#&#8203;1144](https://redirect.github.com/klauspost/compress/pull/1144)\n\n**Full Changelog**:\n<https://github.com/klauspost/compress/compare/v1.18.5...v1.18.6>\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: (UTC)\n\n- Branch creation\n  - \"before 8am every weekday\"\n- Automerge\n  - At any time (no schedule defined)\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n👻 **Immortal**: This PR will be recreated if closed unmerged. Get\n[config\nhelp](https://redirect.github.com/renovatebot/renovate/discussions) if\nthat's undesired.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My4xNTkuMiIsInVwZGF0ZWRJblZlciI6IjQzLjE1OS4yIiwidGFyZ2V0QnJhbmNoIjoibWFpbiIsImxhYmVscyI6WyJkZXBlbmRlbmNpZXMiXX0=-->\n\n---------\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: otelbot <197425009+otelbot@users.noreply.github.com>",
          "timestamp": "2026-05-05T22:21:05Z",
          "tree_id": "351e4b7415aaad326fafd205990c3385cf037324",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/43b48f2f0173874c81756d913b4cef9cd9a58c25"
        },
        "date": 1778026840403,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.8547009229660034,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 5.75180746704229,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 6.206558848564575,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.9078125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.03515625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 5989.7163737045485,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 6040.910530744758,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.006848,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 213548.01947030422,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 176601.55405596356,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
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
          "id": "78856dcb2ecd93270265296c7c279cd9ab877e24",
          "message": "feat(otap-dataflow): Add stopwatch `signals.incoming` and `signals.outgoing` metrics (#2839)\n\n# Change Summary\n\nFollow-up to #2747\n\nAdds two MMSC metrics on the existing `stopwatch` metric set so\noperators can compare signal volume in vs. out across a stopwatch range,\nalongside the existing combined compute duration. \"Signals\" here means\nindividual log records, spans, or metric data points\n(`OtapPdata::num_items()`).\n\n| Metric | Recorded | Why |\n|---|---|---|\n| `stopwatch.signals.incoming` | At the start node, **before**\n`process()` runs | Any filter/drop in the start processor itself does\nnot undercount entry volume. |\n| `stopwatch.signals.outgoing` | At the stop node, **after** `process()`\ncompletes | Reflects what actually leaves the range. |\n\nImplemented as two metric set types (`StopwatchStartMetrics`,\n`StopwatchStopMetrics`) sharing one entity per stopwatch — mirrors the\n`ChannelSenderMetrics` / `ChannelReceiverMetrics` precedent. Each role\nregisters its own `MetricSet` against the same entity and drains its own\naccumulator on the periodic `CollectTelemetry` tick and at shutdown.\n\nTo capture the incoming count pre-process, the existing\n`ProcessorSendHook` trait is renamed to `FlowMeasurementHook` and gains\na default-no-op `after_processor_receive` method. The engine run loops\n(Local + Shared) call it immediately after `inbox.recv_when(...)`\nreturns a `Message::PData`, before `begin_process_timing` and\n`process()`. `OtapPdata` overrides it to drive the start-side counter;\ntest PData stand-ins (`()`, `String`, `TestMsg`) get blanket no-op\nimpls.\n\nThe two hooks fire from different surfaces by design, matching the\nasymmetric flow control of a processor:\n\n| Hook | Fires from | Cardinality per `process()` | Captures |\n|---|---|---|---|\n| `after_processor_receive` | Engine run loop | Exactly 1 (1 dequeue per\niteration) | True pre-process input volume |\n| `before_processor_send` | Effect handler `send_message[_to]` | 0..N\n(drop, pass-through, or fan out) | What actually leaves |\n\n**Behavior change:** removed the `PROCESS_DURATION` gate in\n`build_stopwatch_state`. Stopwatches are already explicit opt-in via the\ntelemetry policy YAML; the gate was redundant and signal counts don't\nneed the timing path. Pipelines with stopwatches under `runtime_metrics:\nbasic`/`none` will now run them instead of silently skipping.\n\n## Demo\n\n`configs/fake-stopwatch-demo.yaml` now includes a 1-in-3\n`processor:log_sampling` node inside the stopwatch range so\n`signals.outgoing` is visibly smaller than `signals.incoming`.\n\n```bash\ncargo run --bin df_engine -- --config configs/fake-stopwatch-demo.yaml\ncurl -s 'http://127.0.0.1:8080/api/v1/telemetry/metrics?format=json' \\\n  | jq '.metric_sets[] | select(.name == \"stopwatch\")'\n```\n\nSample output (truncated, after ~38 collection cycles at 10\nsignals/sec):\n\n```json\n{\n  \"name\": \"stopwatch.signals.incoming\",\n  \"value\": { \"min\": 10.0, \"max\": 10.0, \"sum\": 380.0, \"count\": 38 }\n}\n{\n  \"name\": \"stopwatch.compute.duration\",\n  \"value\": { \"min\": 2859829.0, \"max\": 6619768.0, \"sum\": 170014602.0, \"count\": 38 }\n}\n{\n  \"name\": \"stopwatch.signals.outgoing\",\n  \"value\": { \"min\": 3.0, \"max\": 4.0, \"sum\": 127.0, \"count\": 38 }\n}\n```\n\nReading: 380 signals entered the range (38 batches × 10 signals), 127\nleft it (≈1/3, matching the sampler ratio), and `compute.duration`\naverages ~4.47 ms per batch across the chain (170014602 ns / 38). Both\nsignal-count metrics share the same `stopwatch.name` / `start_node` /\n`stop_node` attributes as the duration metric, so they correlate without\njoins.\n\n## What issue does this PR close?\n\n* Related to #2782 \n* Closes #2837 \n\n## How are these changes tested?\n\nUnit Tests / Local runs\n\n## Are there any user-facing changes?\n\n1. Stopwatch duration metric will now be tracked and emitted even on\n`runtime_metrics: basic/none`.\n2. New Stopwatch metrics for `consumed` and `produced`",
          "timestamp": "2026-05-05T23:00:46Z",
          "tree_id": "d012eaced09e897594792ffe8c20e93222c0c231",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/78856dcb2ecd93270265296c7c279cd9ab877e24"
        },
        "date": 1778027887590,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.2729843854904175,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 5.779009317539973,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 6.174735050669143,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.4953125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.78125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 6031.150570130537,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 6107.926178519751,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.019062,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 214427.07831625224,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 177503.78557134996,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
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
          "id": "7615259cc149f767ce7d03e179311e07e5bfe2e2",
          "message": "Add node-local delayed resume to ProcessorInbox (#2471)\n\n# Change Summary\n\nAdd processor-local delayed `pdata` resume via\n`EffectHandler::requeue_later(...)`, integrated into `ProcessorInbox`,\nand migrate `retry_processor` from the old global `delay_data(...)`\npath.\n\nThis PR completes the delayed-payload side of the node-local scheduling\nwork. The existing wakeup path remains for lightweight timer-style\ncallbacks, while `requeue_later(...)` is for retry-style retained\npayloads that must be delivered back to the same processor later.\n\nThe goal is to move retry-style deferred work out of the global runtime\npath and into node-local inbox orchestration, while keeping the old\nglobal delayed-data plumbing in place temporarily for compatibility.\nThat keeps this PR focused on local delayed resume and retry migration\nonly; the cleanup and removal of the old global path happens in the next\nPR.\n\n## What Changed\n\n- Added a bounded processor-local delayed-resume queue in the node-local\nscheduler.\n- Exposed `EffectHandler::requeue_later(when, data)` with\npayload-preserving rejection.\n- Delivered due local delayed resumes through `ProcessorInbox` as\n`NodeControlMsg::DelayedData`.\n- Preserved inbox control/pdata fairness for delayed resumes.\n- On processor shutdown, pending delayed resumes are surfaced\nimmediately for drain; new delayed resumes are rejected.\n- Migrated `retry_processor` to schedule retries with\n`requeue_later(...)`.\n\n## Benchmark\n\nResults based on a local Criterion benchmark.\n\n| Scenario | `delay_data` | `requeue_later` | Result |\n| --- | ---: | ---: | ---: |\n| Schedule only | ~1.247 ms / 10k | ~286.9 us / 10k | ~4.35x faster |\n| Schedule + due delivery | ~2.720 ms / 10k | ~1.911 ms / 10k | ~1.42x\nfaster |\n\nThe new path removes the runtime-control scheduling hop for retry-style\nretained pdata and keeps delayed resume state local to the processor\ninbox/scheduler.\n\n## What issue does this PR close?\n\n* Part of #2465\n\n## How are these changes tested?\n\n- Added scheduler tests for delayed resume ordering, capacity,\npayload-preserving rejection, and shutdown drain conversion.\n- Added inbox tests for delayed-resume delivery, fairness, shutdown\nrejection, and shutdown drain behavior.\n- Updated retry tests for local delayed retry and terminal NACK behavior\nwhen `requeue_later(...)` rejects.\n- Ran targeted engine checks/tests for the scheduler and inbox behavior.\n- `cargo xtask check`\n\n## Are there any user-facing changes?\n\nNo user-facing changes.\n\nThis PR is an internal runtime change only.",
          "timestamp": "2026-05-06T03:13:41Z",
          "tree_id": "5dbf69cbd93a5c6cd0f6c83a4febda12d9578c9e",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/7615259cc149f767ce7d03e179311e07e5bfe2e2"
        },
        "date": 1778039890968,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.9887005090713501,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 5.803074946238963,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 6.320074280408543,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.043619791666668,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.1171875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 6041.250513657785,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 6100.980391617679,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003471,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 214399.87059795065,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 177408.6571240466,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
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
          "id": "ed9830013345f296033c4c29d53917561622a5cf",
          "message": "refactor(perf-test): factor loadgen flush_batch_metrics into _BatchMetricsAccumulator (#2847)\n\nFollow-up to #2822 addressing review feedback from @jmacd — the\nper-thread accumulator and flush logic was duplicated across all three\nworker functions. Extract it into a small `_BatchMetricsAccumulator`\nhelper class (with `__slots__`) so each worker is just `acc =\n_BatchMetricsAccumulator(self); …; acc.flush()`.\n\nBehavior unchanged. Smoke-tested with the rebuilt loadgen container —\n`/metrics` counters tick up at the expected ~5000 logs/sec during a\nsyslog UDP run, same as before. Net diff: `+58 / -84`.",
          "timestamp": "2026-05-06T15:00:46Z",
          "tree_id": "d096833928a9fb6b09c912bf7096d31e79ffbd99",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/ed9830013345f296033c4c29d53917561622a5cf"
        },
        "date": 1778083189489,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.8335683345794678,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 5.709314579388946,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 6.180060535506403,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.831640625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.75390625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 6049.429986273596,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5938.509549289735,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.006976,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 214047.67800631875,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 176852.93284520117,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
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
          "id": "ce62f84dfe379eb94f3dcef123d79d3c69c5abf6",
          "message": "chore: Rename `stopwatches` to `flow_metrics` to better match supported metrics (#2846)\n\n# Change Summary\n\n## Motivation\n\nThe original `stopwatch` feature measured aggregate per-message compute\nduration across a contiguous range of processor nodes. As item-count\nmetrics were added in #2839 at the start and end of each range, the\n\"stopwatch\" name became misleading — the feature now records three\ndistinct measurements over a flow. The rename reframes the feature\naround the *flow* it observes and lets operators opt into individual\nmeasurements.\n\n## What changed\n\n- Terminology: \"stopwatch\" → \"flow_metrics\"; `stop_node` → `end_node`.\n- Config: `telemetry.stopwatches` → `telemetry.flow_metrics`, with\n`start_node`/`end_node` nested under `bounds`, and a new optional\n`metrics` selector (`compute_duration`, `signals_incoming`,\n`signals_outgoing`). Omitting `metrics` enables all three.\n- Metrics emitted on the `flow` metric set:\n  - `flow.compute.duration` (ns)\n  - `flow.signals.incoming` (items, at the start node)\n  - `flow.signals.outgoing` (items, at the end node)\n\nRuntime measurement semantics are unchanged: per-message wall-clock time\ninside `process()` is accumulated via the `Instant` send-marker advanced\non every `send_message`.\n\n## Example\n\n```yaml\ntelemetry:\n  flow_metrics:\n    - name: ingest_pipeline\n      bounds:\n        start_node: sampler\n        end_node: attr4\n      # optional; omit to enable all\n      metrics: [compute_duration, signals_incoming, signals_outgoing]\n```\n\nThis config structure is flexible for additional improvements:\n\n- Allowing declaration of bounds using node labels (instead of node\nname)\n- Easily extendable for `messages_incoming` and `messages_outgoing`\nmetrics\n\n## What issue does this PR close?\n\n* Closes #2845\n\n## How are these changes tested?\n\nUnit test / local runs\n\n## Are there any user-facing changes?\n\nYes, config contract for `stopwatches` becomes `flow_metrics`",
          "timestamp": "2026-05-06T16:03:31Z",
          "tree_id": "fef1b4c6f2dc91b46c77b898853ca31f3d165504",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/ce62f84dfe379eb94f3dcef123d79d3c69c5abf6"
        },
        "date": 1778091086978,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.7163323760032654,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 5.771186580759733,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 6.245991952955741,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.54296875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.98828125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 5955.708517516767,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5998.371185980354,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.005623,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 214234.6489522317,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 177301.25791203545,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "williamcandlerbutler@gmail.com",
            "name": "Will Butler",
            "username": "wbutler"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "e6e929f935ae98bb1e7054bad7d2fe20d3cd1aaa",
          "message": "Add query-engine functions `upper_case` and `lower_case` (#2849)\n\nCloses: #2821 \n\nThis is my first contribution to the project. Guidance and corrections,\nboth w.r.t. the code itself and w.r.t. the form and practice of my\ncontribution, are very welcome.\n\nThis change connects the Datafusion `upper_case` and `lower_case` user\ndefined funcs into the query engine via the existing\n`InvokeFunctionExpr` path. For each function, adds:\n\n- A function-name constant in `consts.rs`\n- A parser registration extending `parser.rs::default_parser_options`\n- A `from_func_name` clause in `DataFusionFunctionDef` (`expr.rs`)\nreturning `ExprLogicalType::String`\n\nExample queries that this change enables:\n\n```\nlogs | extend attributes[\"x\"] = upper_case(attributes[\"x\"])\nlogs | extend attributes[\"x\"] = lower_case(attributes[\"x\"])\n```\n\n## Tests\n\n- In `expr.rs`, adds a unit test for each new function that directly\nbuilds the `InvokeFunctionScalarExpression`, builds a synthetic record\nbatch, executes the func against the data, and compares to a result.\nFollowed the pattern in `test_function_Invocation_sha256`.\n- In `assign.rs`, adds an E2E assign test that covers overwriting\n`attributes[\"...\"]` via `extend`. Invoked via both `OplParser` and\n`KqlParser`. General structure borrowed from\n`test_update_attr_to_hash_function_call_result_all_supported_types`.\n\n## Validation\n\nAll of these commands pass cleanly.\n\n- `cargo check -p otap-df-query-engine`\n- `cargo test -p otap-df-query-engine` All OK (581 passed)\n- `cargo clippy -p otap-df-query-engine --all-targets -- -D warnings`\n- `cargo fmt --all -- --check`\n- `cargo xtask quick-check`\n\n## Notes\n\nThis will initially fail the CLA check. My reading of current docs are\nto allow this to happen so I may sign it as part of this PR.\n\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-05-06T17:52:14Z",
          "tree_id": "52f748032f6b39dd7c316c618cbcb8a56ddbd04a",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/e6e929f935ae98bb1e7054bad7d2fe20d3cd1aaa"
        },
        "date": 1778092648434,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.8335683345794678,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 5.752768403224812,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 6.350746361102509,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.879036458333335,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.0859375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 6049.787285500603,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5938.860297191001,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003432,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 212779.99727173668,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 175786.43582413826,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "nonicked@protonmail.com",
            "name": "Nick Nikolakakis",
            "username": "nicknikolakakis"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "fd024d61e05c973c86696b2d8d0d4b49cf97cd43",
          "message": "feat(query-engine): add uuid() and uuidv7() scalar functions (#2853)\n\n# Change Summary\n\nAdd two zero-arg scalar functions in the OTAP query-engine, per #2833:\n\n- `uuid()`: UUID v4 string per row, backed by DataFusion's built-in\n[`string::uuid`](https://datafusion.apache.org/user-guide/sql/scalar_functions.html#uuid)\nUDF.\n- `uuidv7()`: UUID v7 string per row, implemented as a custom volatile\n`ScalarUDF` (DataFusion has no v7 equivalent) using\n`uuid::Uuid::now_v7`.\n\n```kql\nlogs | extend attributes[\"my.log.id\"] = uuid()\nlogs | extend attributes[\"my.log.id\"] = uuidv7()\n```\n\n## Zero-arg function support\n\n`uuid()` / `uuidv7()` are the first 0-arg functions in the engine, so\nthis also lifts the prior `\"Only functions with one or more arguments\ncurrently supported\"` guard in\n`ExprLogicalPlanner::plan_function_invocation`.\n\nZero-arg invocations now plan with `DataScope::Root` (instead of\n`DataScope::StaticScalar`). This is important for volatile UDFs:\n`Volatility::Volatile` UDFs like `uuid()` produce one new value per row\nin the input batch, but evaluating them in `StaticScalar` scope would\nfeed them the empty placeholder batch (or collapse the result into a\n`ColumnarValue::Scalar` that gets broadcast to every row, so all rows\nwould get the same UUID). Routing through `Root` makes the function\nevaluate against the root batch and produce per-row distinct UUIDs.\n\nThis pattern should generalize to other zero-arg volatile functions\n(e.g. `now()` mentioned in the original TODO) and to non-volatile\nzero-arg functions, since broadcasting an identical scalar across\n`num_rows` rows is equivalent to producing `num_rows` identical values.\n\n## Files\n\n- `consts.rs`, `parser.rs`: register `uuid` and `uuidv7` as external\nfunctions with zero param placeholders.\n- `pipeline/expr.rs`: import DataFusion's `uuid()` and the new\n`uuidv7()` UDF, add them to `DataFusionFunctionDef::from_func_name`, and\nhandle the zero-arg path in `plan_function_invocation`.\n- `pipeline/functions/uuidv7.rs` (new): custom `UuidV7Func` mirroring\nDataFusion's `UuidFunc` shape but using `Uuid::now_v7`.\n- `pipeline/functions.rs`: wire the new module via `make_udf_function!`.\n- `crates/query-engine/Cargo.toml`: depend on the workspace `uuid` crate\n(already pulled in transitively, just needs to be declared).\n\n## What issue does this PR close?\n\n* Closes https://github.com/open-telemetry/otel-arrow/issues/2833\n* Part of #2818\n\n## How are these changes tested?\n\nThree layers of tests, all in this PR:\n\n- `pipeline::functions::uuidv7` UDF tests: verify per-row uniqueness,\nzero-row case, argument rejection, and that produced UUIDs parse as v7.\n- `pipeline::expr` planner tests (`test_function_invocation_uuid_v4`,\n`test_function_invocation_uuid_v7`): drive\n`ExprLogicalPlanner::plan_scalar_expr` with a 0-arg\n`InvokeFunctionScalarExpression`, execute against a 3-log OTAP batch,\nand check the resulting `StringArray` contains 3 distinct UUIDs of the\nexpected version.\n- `pipeline::assign` end-to-end tests\n(`test_set_attr_to_uuid_v{4,7}_function_call_result_{opl,kql}_parser`):\nrun `logs | extend attributes[\"my.log.id\"] = uuid()` (and `uuidv7()`)\nthrough both OPL and KQL parsers and the full pipeline, assert each log\nrow receives its own distinct UUID of the right version.\n\n`cargo xtask check` passes locally for the affected crates. There is one\nunrelated network-dependent failure in `otap-df-core-nodes`\n(`fake_data_generator` tests cloning\n`open-telemetry/semantic-conventions` over Git) that occurs on my\nmachine regardless of these changes.\n\n## Are there any user-facing changes?\n\nYes: users of the transform processor / query-engine can now invoke\n`uuid()` and `uuidv7()` in OPL/KQL programs, primarily for assigning a\nunique identifier per record:\n\n```kql\nlogs | extend attributes[\"my.log.id\"] = uuid()\nlogs | extend attributes[\"trace.log.id\"] = uuidv7()\n```\n\n---------\n\nSigned-off-by: Nick Nikolakakis <nonicked@protonmail.com>\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-05-06T20:48:09Z",
          "tree_id": "89c7db5b85cc5183ecae24a671ba6dc7eaa0fc6e",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/fd024d61e05c973c86696b2d8d0d4b49cf97cd43"
        },
        "date": 1778103954594,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.2729843854904175,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 5.740114800437457,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 6.216619761867944,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.360026041666668,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.7578125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 6032.718277186159,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 6109.513842242277,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003465,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 213269.85455366212,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 176107.67298714045,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
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
          "id": "3888bfead60befcc47f31ceff8a9ead7ce1de33a",
          "message": "Update pipeline perf python dependencies (#2719)\n\nThis PR contains the following updates:\n\n| Package | Change |\n[Age](https://docs.renovatebot.com/merge-confidence/) |\n[Confidence](https://docs.renovatebot.com/merge-confidence/) |\n|---|---|---|---|\n|\n[opentelemetry-exporter-otlp](https://redirect.github.com/open-telemetry/opentelemetry-python)\n| `==1.41.0` → `==1.41.1` |\n![age](https://developer.mend.io/api/mc/badges/age/pypi/opentelemetry-exporter-otlp/1.41.1?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/pypi/opentelemetry-exporter-otlp/1.41.0/1.41.1?slim=true)\n|\n|\n[opentelemetry-proto](https://redirect.github.com/open-telemetry/opentelemetry-python)\n| `==1.41.0` → `==1.41.1` |\n![age](https://developer.mend.io/api/mc/badges/age/pypi/opentelemetry-proto/1.41.1?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/pypi/opentelemetry-proto/1.41.0/1.41.1?slim=true)\n|\n|\n[opentelemetry-sdk](https://redirect.github.com/open-telemetry/opentelemetry-python)\n| `==1.41.0` → `==1.41.1` |\n![age](https://developer.mend.io/api/mc/badges/age/pypi/opentelemetry-sdk/1.41.1?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/pypi/opentelemetry-sdk/1.41.0/1.41.1?slim=true)\n|\n| [pydantic](https://redirect.github.com/pydantic/pydantic)\n([changelog](https://docs.pydantic.dev/latest/changelog/)) | `==2.13.2`\n→ `==2.13.3` |\n![age](https://developer.mend.io/api/mc/badges/age/pypi/pydantic/2.13.3?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/pypi/pydantic/2.13.2/2.13.3?slim=true)\n|\n\n---\n\n### Release Notes\n\n<details>\n<summary>open-telemetry/opentelemetry-python\n(opentelemetry-exporter-otlp)</summary>\n\n###\n[`v1.41.1`](https://redirect.github.com/open-telemetry/opentelemetry-python/releases/tag/v1.41.1):\nVersion 1.41.1/0.62b1\n\n[Compare\nSource](https://redirect.github.com/open-telemetry/opentelemetry-python/compare/v1.41.0...v1.41.1)\n\nThis is a patch release on the previous 1.41.0/0.62b0 release, fixing\nthe issue(s) below.\n\n</details>\n\n<details>\n<summary>pydantic/pydantic (pydantic)</summary>\n\n###\n[`v2.13.3`](https://redirect.github.com/pydantic/pydantic/blob/HEAD/HISTORY.md#v2133-2026-04-20)\n\n[Compare\nSource](https://redirect.github.com/pydantic/pydantic/compare/v2.13.2...v2.13.3)\n\n[GitHub\nrelease](https://redirect.github.com/pydantic/pydantic/releases/tag/v2.13.3)\n\n##### What's Changed\n\n##### Fixes\n\n- Handle `AttributeError` subclasses with `from_attributes` by\n[@&#8203;Viicos](https://redirect.github.com/Viicos) in\n[#&#8203;13096](https://redirect.github.com/pydantic/pydantic/pull/13096)\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: (UTC)\n\n- Branch creation\n  - \"before 8am every weekday\"\n- Automerge\n  - At any time (no schedule defined)\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n👻 **Immortal**: This PR will be recreated if closed unmerged. Get\n[config\nhelp](https://redirect.github.com/renovatebot/renovate/discussions) if\nthat's undesired.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My4xMjMuOCIsInVwZGF0ZWRJblZlciI6IjQzLjEzOS43IiwidGFyZ2V0QnJhbmNoIjoibWFpbiIsImxhYmVscyI6WyJkZXBlbmRlbmNpZXMiXX0=-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: Drew Relmas <drewrelmas@gmail.com>",
          "timestamp": "2026-05-06T21:44:20Z",
          "tree_id": "ac669dd6b4ea9a57f18fcb9bd41c3ec883ee4570",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/3888bfead60befcc47f31ceff8a9ead7ce1de33a"
        },
        "date": 1778107223665,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.5641748905181885,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 5.738721863130465,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 6.252208034880099,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.23125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.38671875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 6049.251654904631,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 6083.3800140296225,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.008745,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 212723.22163869732,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 175814.9047434903,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "86380966+Cloud-Architect-Emma@users.noreply.github.com",
            "name": "EMMANUELA OPURUM",
            "username": "Cloud-Architect-Emma"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "3ab5efb81801f555a13b4db880b5204aa4b4ec43",
          "message": "feat: support format_datetime scalar function in OPL/OTAP query engine (#2850)\n\nFixes #2835\n\n## Summary\nAdds support for `format_datetime(timestamp, format)` as a scalar\nfunction\nin the OPL/OTAP query engine, allowing queries like:\nlogs | set attributes[\"date\"] = format_datetime(timestamp_unix_nano,\n\"%m/%d/%Y\")\n\n## Implementation\nThis is implemented using DataFusion's built-in `to_char` function,\nwhich\nuses chrono strftime format strings, compatible with OTTL's FormatTime\nformats.\n\n## Changes\n- Added `datetime_expressions` feature to datafusion dependency in\n`Cargo.toml`\n- Added `FORMAT_DATETIME_FUNC_NAME` constant to `consts.rs`\n- Registered `format_datetime` as an external function with 2 parameters\nin `parser.rs`\n- Wired `format_datetime` to DataFusion's `to_char` UDF in `expr.rs\n\n---------\n\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-05-06T22:30:06Z",
          "tree_id": "f49948065ecf317022df3adfcfdf4cb8912172a5",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/3ab5efb81801f555a13b4db880b5204aa4b4ec43"
        },
        "date": 1778114227842,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.5617977380752563,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 5.740325369684416,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 6.107425750212142,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.841796875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.01171875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 6075.389972543385,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 6041.258568203254,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003391,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 211756.67606284408,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 174798.22409751598,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
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
          "distinct": false,
          "id": "05a66cbd8eb9024d9601cc5a273ec894b7e9e066",
          "message": "docs: encourage small, single-purpose PRs in CONTRIBUTING.md (#2864)\n\nAdds a \"Keep Pull Requests Small and Incremental\" section to\nCONTRIBUTING.md to set explicit expectations for small, single-purpose\nPRs.",
          "timestamp": "2026-05-07T01:36:40Z",
          "tree_id": "61afd77cbaeb5c822b7e0ad4605c9a18cf087bef",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/05a66cbd8eb9024d9601cc5a273ec894b7e9e066"
        },
        "date": 1778122048827,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.2729843854904175,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 5.734240654239673,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 6.234378579167312,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.453385416666666,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.58984375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 6032.723505244609,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 6109.519136853099,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003413,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 211985.31704070853,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 175069.4043671582,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "AaronRM@users.noreply.github.com",
            "name": "Aaron Marten",
            "username": "AaronRM"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "1d8da10758f5a8bc088b36d6dc217672a4384f08",
          "message": "test: ensure OTLP HTTP server is ready before connecting to avoid timeouts/NACKs (#2854)\n\n# Change Summary\n\nEnsure that the OTLP HTTP endpoint is ready prior to running tests to\navoid timeouts/NACKs causing test failures.\n\n## What issue does this PR close?\n\n* Addresses flaky test failure (test-level 60s timeout) for\n`otap-df-core-nodes::exporters::otlp_http_exporter::test::test_tls_mtls_success_cert_file`\nas reported in #2720\n\n## How are these changes tested?\n\n- Validated that test(s) pass locally on re-run\n\n## Are there any user-facing changes?\n\nNo. This is a test only change.",
          "timestamp": "2026-05-07T01:37:24Z",
          "tree_id": "4a669814b95361668698c00dea44f7133ede43a0",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/1d8da10758f5a8bc088b36d6dc217672a4384f08"
        },
        "date": 1778128370882,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.14104372262954712,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 5.708036375264736,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 5.943565123122193,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.75703125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.2421875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 6049.708139679583,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 6041.175406055211,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004217,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 213554.24908389672,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 176815.08950974856,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
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
          "id": "04351ee2c13fbb7369baf62fd14e0d654ccd6535",
          "message": "refactor(core-nodes): rename fake_data_generator module to traffic_generator (#2872)\n\n# Change Summary\n\nRenames the `fake_data_generator` receiver module to `traffic_generator`\nso that the module folder and Rust symbols match the URN id\n(`urn:otel:receiver:traffic_generator`). This brings the receiver in\nline with the convention used by every other receiver/processor/exporter\nin `core-nodes`, where the module name matches the URN id segment.\n\nThis was [flagged in PR #1857 review\nfeedback](https://github.com/open-telemetry/otel-arrow/pull/1857#discussion_r2718063522).\nThe URN was last touched by\n[#1948](https://github.com/open-telemetry/otel-arrow/pull/1948) and\n[#2110](https://github.com/open-telemetry/otel-arrow/pull/2110), but the\nmodule folder and Rust symbol names were not realigned at the time.\n\n## Scope\n\nTo keep this PR focused, the public observability surface (metric set\nname, log event names, semconv scope, `FakeSignalReceiverMetrics` type)\nis intentionally **not** renamed here. A follow-up PR will rename the\nmetrics and related strings.\n\n## How are these changes tested?\n\n`cargo check`, the renamed module's unit tests, and the otap integration\n/ validation tests that depend on it all pass locally.\n\n## Are there any user-facing changes?\n\nThe Rust module path and the URN `pub const` constant are renamed. The\nURN string value itself is unchanged, so YAML/config consumers are\nunaffected.",
          "timestamp": "2026-05-07T02:45:17Z",
          "tree_id": "36c2a7b59d42859c9fbf57158433f3edd07aed95",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/04351ee2c13fbb7369baf62fd14e0d654ccd6535"
        },
        "date": 1778129403026,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -2.4390242099761963,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 5.744422412242586,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 6.300994475138122,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.702734375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.09765625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 5946.650349193906,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 6091.690601613269,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.010927,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 212772.25281710227,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 175763.91051858396,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
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
          "id": "5dd7b4fd05fa2a4103bc67563077db6289b94457",
          "message": "chore(deps): update dependency pydantic to v2.13.4 (#2876)\n\nThis PR contains the following updates:\n\n| Package | Change |\n[Age](https://docs.renovatebot.com/merge-confidence/) |\n[Confidence](https://docs.renovatebot.com/merge-confidence/) |\n|---|---|---|---|\n| [pydantic](https://redirect.github.com/pydantic/pydantic)\n([changelog](https://docs.pydantic.dev/latest/changelog/)) | `==2.13.3`\n→ `==2.13.4` |\n![age](https://developer.mend.io/api/mc/badges/age/pypi/pydantic/2.13.4?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/pypi/pydantic/2.13.3/2.13.4?slim=true)\n|\n\n---\n\n### Release Notes\n\n<details>\n<summary>pydantic/pydantic (pydantic)</summary>\n\n###\n[`v2.13.4`](https://redirect.github.com/pydantic/pydantic/releases/tag/v2.13.4):\n2026-05-06\n\n[Compare\nSource](https://redirect.github.com/pydantic/pydantic/compare/v2.13.3...v2.13.4)\n\n#### v2.13.4 (2026-05-06)\n\n##### What's Changed\n\n##### Packaging\n\n- Bump libc from 0.2.155 to 0.2.185 by\n[@&#8203;Viicos](https://redirect.github.com/Viicos) in\n[#&#8203;13109](https://redirect.github.com/pydantic/pydantic/pull/13109)\n- Adapt `pydantic-core` linker flags on macOS by\n[@&#8203;washingtoneg](https://redirect.github.com/washingtoneg) and\n[@&#8203;Viicos](https://redirect.github.com/Viicos) in\n[#&#8203;13147](https://redirect.github.com/pydantic/pydantic/pull/13147)\n\n##### Fixes\n\n- Preserve `RootModel` core metadata by\n[@&#8203;Viicos](https://redirect.github.com/Viicos) in\n[#&#8203;13129](https://redirect.github.com/pydantic/pydantic/pull/13129)\n-\n\n**Full Changelog**:\n<https://github.com/pydantic/pydantic/compare/v2.13.3...v2.13.4>\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: (UTC)\n\n- Branch creation\n  - \"before 8am every weekday\"\n- Automerge\n  - At any time (no schedule defined)\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My4xNTkuMiIsInVwZGF0ZWRJblZlciI6IjQzLjE1OS4yIiwidGFyZ2V0QnJhbmNoIjoibWFpbiIsImxhYmVscyI6WyJkZXBlbmRlbmNpZXMiXX0=-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2026-05-07T04:28:00Z",
          "tree_id": "537cdcf960047a0fcf8df4fef3fb91a1a203a5af",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/5dd7b4fd05fa2a4103bc67563077db6289b94457"
        },
        "date": 1778141313058,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.42432817816734314,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 5.779165914655873,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 6.25017484140492,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.663932291666665,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.9921875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 6032.7253149592625,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 6007.126763410637,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003395,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 214239.6867884831,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 177557.28056815048,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
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
          "distinct": false,
          "id": "e3eee4c20c65a6f65ff8f39eb86037d9eb482f63",
          "message": "Clarify syslog CEF invalid log metric (#2880)\n\nUpdates the syslog CEF receiver's received_logs_invalid metric\ndescription to clarify that it counts zero-length payloads rejected by\nthe parser.\n\nCloses #2285\n\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-05-07T11:52:43Z",
          "tree_id": "1099552f75a61d0b8de7c6a66b927c3ec507337c",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/e3eee4c20c65a6f65ff8f39eb86037d9eb482f63"
        },
        "date": 1778157713950,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.1299434900283813,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 5.742026642331702,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 6.081312185686654,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.590885416666666,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.84375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 6041.257057974342,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 6109.519849589872,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003406,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 212463.23847843413,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 175604.82580539634,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "42037371+guancioul@users.noreply.github.com",
            "name": "Kuan-Hao (Michael) Lai",
            "username": "guancioul"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "fbf013edfb32b5c1f4922f1cfdf1ecad10dfa2a8",
          "message": "Add ltrim and rtrim functions to query engine (#2851)\n\n# Change Summary\n\nAdd ltrim and rtrim functions to the query engine\n\n## What issue does this PR close?\n\n* Closes #2820\n\n## How are these changes tested?\n\nAdded 4 unit tests in pipeline/assign.rs covering both ltrim and rtrim\nwith OPL and KQL parsers.\n\n## Are there any user-facing changes? \n\nYes. Users can now use ltrim and rtrim in OPL queries: \n\n`logs | set attributes[\"x\"] = ltrim(attributes[\"x\"], \" \")`\n`logs | set attributes[\"y\"] = rtrim(attributes[\"y\"], \"\\n\")`\n\n---------\n\nSigned-off-by: guancioul <guancioul@gmail.com>",
          "timestamp": "2026-05-07T12:06:03Z",
          "tree_id": "4a911e941e47c03b7302307047225c3bb53df39f",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/fbf013edfb32b5c1f4922f1cfdf1ecad10dfa2a8"
        },
        "date": 1778164993424,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.9943019151687622,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 5.79887618949945,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 6.237832985063076,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.707421875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.9765625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 5990.029516674394,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 6109.488794784709,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003711,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 213199.01405586317,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 176396.2749829796,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
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
          "distinct": false,
          "id": "17637ee164e0265d4479d9da79ff0501a7791014",
          "message": "refactor(core-nodes): rename traffic generator metrics (#2886)\n\nRenames the traffic generator receiver observability surface from the\nold fake-data-generator naming to traffic-generator naming, following up\non #2872.\n\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-05-07T15:19:40Z",
          "tree_id": "b6afbf281224de0b793068754e8ab2daf8c8505a",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/17637ee164e0265d4479d9da79ff0501a7791014"
        },
        "date": 1778170835160,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.1299434900283813,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 5.769706476489993,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 6.318483324305502,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.250260416666666,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.203125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 6041.25293001917,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 6109.515674991138,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003447,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 212881.53141394074,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 175977.7842170306,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "33842784+JakeDern@users.noreply.github.com",
            "name": "Jake Dern",
            "username": "JakeDern"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "be41f5c64fdbbb51192132d3e3590f692ea4fb34",
          "message": "feat: Initial comparison dashboard tools (#2865)\n\n# Change Summary\n\nThis PR contains the initial comparison dashboard skeleton. No data,\ncomparisons, templates, or anything else is included yet. This is purely\nthe scaffolding.\n\n## What issue does this PR close?\n\n* Closes #2856\n\n## How are these changes tested?\n\nQuick sanity check locally: \n\n<img width=\"3824\" height=\"521\" alt=\"image\"\nsrc=\"https://github.com/user-attachments/assets/ae718576-0c0f-406b-a4b6-55e4760d380e\"\n/>\n\n## Are there any user-facing changes?\n\nNo.\n\n---------\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2026-05-07T16:36:04Z",
          "tree_id": "f975f8f38b2677365544d6a0e43616b1e0c5a0fa",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/be41f5c64fdbbb51192132d3e3590f692ea4fb34"
        },
        "date": 1778179221804,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.5647226572036743,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 5.543681469335306,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 6.548199055800634,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.523958333333333,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.23828125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 5998.554324675919,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 6092.415060908402,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003791,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 211847.22694557812,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 174855.23957398444,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
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
          "id": "4857f2baa0ccebb76a6c379df75e8a80da7170cb",
          "message": "chore: Rename flow_metric to flow_metrics and remove extra instrument name (#2896)\n\n# Change Summary\n\nAddress small comment here:\nhttps://github.com/open-telemetry/otel-arrow/issues/2859#issuecomment-4393428306\nas well as following pattern observed in #2888 to simplify instrument\nnames. The extra `flow.` prefix was unnecessary.\n\nThe customer-facing config was already `flow_metrics`, this PR just\nbrings the code in line with that terminology.\n\n## What issue does this PR close?\n\nN/A, minor rename\n\n## How are these changes tested?\n\n## Are there any user-facing changes?\n\nSlight changes to Flow metric instrument names",
          "timestamp": "2026-05-07T18:28:12Z",
          "tree_id": "8c0064b9ece3e09ee71972b7a54ef153651e7d02",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/4857f2baa0ccebb76a6c379df75e8a80da7170cb"
        },
        "date": 1778185912859,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.1235954761505127,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 5.807713582648565,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 6.547372330547818,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.299088541666666,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.6484375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 6075.303201866642,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 6007.041368137803,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004248,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 213407.57177111215,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 176542.02371734116,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "33842784+JakeDern@users.noreply.github.com",
            "name": "Jake Dern",
            "username": "JakeDern"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "99667f6a529cb345f6ba371cdc8d83620f313f9b",
          "message": "feat(comparison_dashboard): Add DFE orchestrator and step templates (#2871)\n\n# Change Summary\n\nThis is the next PR which adds DFE orchestrator and step templates.\nThese templates define the multi-rate tests and are what I've used\nacross all comparisons so far.\n\nThey're designed to be used with just a single core for the SUT, however\nwe can definitely imagine them being adjusted in the future to be more\ngeneric on that front.\n\nThis is one layer up the stack from and depends on #2870. \n\n## What issue does this PR close?\n\n* Closes #2866 \n\n## How are these changes tested?\n\nThis is a port from my private branch. We will need one more PR after\nthis one to have visible results here, so please bear with me as I'm\ntrying to stack the PR diffs in a way that's easier to review and\nhighlights the dependency chain.\n\n## Are there any user-facing changes?\n\nNo.\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2026-05-07T19:25:20Z",
          "tree_id": "9d62bb27a7d78e549a6e362c505b6037bead8edc",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/99667f6a529cb345f6ba371cdc8d83620f313f9b"
        },
        "date": 1778195462080,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 5.758699100279257,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 6.367623762376237,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.447786458333333,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.6640625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 6074.989450875073,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 6074.989450875073,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.007347,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 211556.60340668095,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 174631.09288212427,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "totan@microsoft.com",
            "name": "Tom Tan",
            "username": "ThomsonTan"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "1ffbbd1e3d7b6f7cbb16bdbb6668064246ccc42d",
          "message": "ci(codecov): set after_n_builds to 4 (matrix-expanded upload count) (#2898)\n\n# Change Summary\n\n`codecov.notify.after_n_builds` was set to 3 but CI actually produces 4\ncoverage uploads. Codecov was therefore posting status as soon as the\n3rd report landed -- often before the slowest job (`coverage`) had\nfinished -- producing the premature/partial status shown in #2890.\n\nThe original miscount treated `experimental_tests` as a single upload,\nbut its `folder` matrix expands to two uploads on `ubuntu-latest`\n(`experimental/query_abstraction`, `experimental/query_engine`).\n\n## What issue does this PR close?\n\n<!--\nWe highly recommend correlation of every PR to an issue\n-->\n\n* Closes #2890 \n\n## How are these changes tested?\n\n## Are there any user-facing changes?\n\n <!-- If yes, provide further info below -->",
          "timestamp": "2026-05-07T19:26:42Z",
          "tree_id": "bbc02e267f8e352ca424909625e9bcc0aa6743cb",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/1ffbbd1e3d7b6f7cbb16bdbb6668064246ccc42d"
        },
        "date": 1778196499506,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.1315417289733887,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 5.817837005908653,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 6.31918916827853,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.451432291666666,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.59375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 6032.38922935621,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 6100.648230537045,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.006738,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 212454.10918673113,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 175628.99867789645,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
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
          "distinct": false,
          "id": "e3ca57241e90ed25bbc95e719744a1853b398756",
          "message": "docs: Expand SIG meeting welcoming language (#2897)\n\n## Changes\n\nThe Contributing section already has welcoming language for the SIG\nmeeting, but is missing the explicit opening that the meeting is open to\nall. Adding two short sentences brings the wording in line with the same\ntext used in\n[opentelemetry-rust](https://github.com/open-telemetry/opentelemetry-rust#contributing),\n[opentelemetry-dotnet](https://github.com/open-telemetry/opentelemetry-dotnet#contributing),\nand\n[weaver](https://github.com/open-telemetry/weaver/blob/main/CONTRIBUTING.md).\n\nPer the discussion in\n[open-telemetry/community#1805](https://github.com/open-telemetry/community/issues/1805),\nexplicit welcoming language on SIG meeting invitations helps lower the\nimplicit gatekeeping that the word \"meeting\" can convey to newcomers.\n\nScope is limited to the welcoming paragraph; the broader \"meetings vs.\noffice hours / public meetings\" naming discussion in community#1805 is\nintentionally out of scope here.",
          "timestamp": "2026-05-07T20:21:38Z",
          "tree_id": "dc706a7ed7f4e7f9e27a3066df9edb256417529d",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/e3ca57241e90ed25bbc95e719744a1853b398756"
        },
        "date": 1778201826056,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.42553189396858215,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 5.780585608622721,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 6.044466197613949,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.559114583333333,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.78515625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 6015.637357327976,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5990.038900488283,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003617,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 211745.46732119063,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 174945.78342199852,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "AaronRM@users.noreply.github.com",
            "name": "Aaron Marten",
            "username": "AaronRM"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "3acddccd49018d593b3ef3159d435a548cb699ae",
          "message": "test: improve test_lazy_reload_resolver polling mechanism to address test flakiness (#2899)\n\n# Change Summary\n\nAddresses flaky `test_lazy_reload_resolver` by changing from a fixed\nduration sleep to instead poll for the certificate to change up to a\nlonger timeout.\n\n## What issue does this PR close?\n\n* Addresses a flaky test report identified via #2720\n\n## How are these changes tested?\n\nValidated that test passes locally through several iterations\n\n## Are there any user-facing changes?\n\nNo. This is a test-only change.",
          "timestamp": "2026-05-07T20:22:13Z",
          "tree_id": "463f7d20ea5f0b576463d40997cd4d8213bd1780",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/3acddccd49018d593b3ef3159d435a548cb699ae"
        },
        "date": 1778202873826,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.5641748905181885,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 5.73351920066812,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 6.167110698537944,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.466145833333332,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.015625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 6049.789201157607,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 6083.920592983603,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003413,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 212559.71592045325,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 175662.91982821948,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "33842784+JakeDern@users.noreply.github.com",
            "name": "Jake Dern",
            "username": "JakeDern"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "0c6c182698606309e88cebc58886a7a07a7ba129",
          "message": "feat(comparison_dashboard): Add comparison logs sql report (#2870)\n\n# Change Summary\n\nThis PR adds the logs report for the comparison dashboard. It's similar\nto the report that we use for the integration tests but includes some\nmodifications like pulling the entire timeseries for all observations.\n\n## What issue does this PR close?\n\n* Closes #2867\n\n## How are these changes tested?\n\nThis is a copy of the file that I've been using for a while on a private\nbranch, we will need a few more changes to demonstrate this end to end.\n\nLogically this file has no dependencies on other files, however other\nfiles depend on it. Hence why it's coming first.\n\n## Are there any user-facing changes?\n\nNo.",
          "timestamp": "2026-05-07T20:44:48Z",
          "tree_id": "6f4f6c2cdc439c557d67939f8abe9c66ec46347c",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/0c6c182698606309e88cebc58886a7a07a7ba129"
        },
        "date": 1778204336321,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.9900990128517151,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 5.792174703350942,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 6.155865585168019,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.259114583333332,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.5390625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 6032.205368276833,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 6091.930173903336,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.008567,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 213764.64307123935,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 176619.0247377201,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "lalit_fin@yahoo.com",
            "name": "Lalit Kumar Bhasin",
            "username": "lalitb"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "3c03f86894725a9157cb11ff00d9234df31d185a",
          "message": "  Add Linux host metrics receiver for OTAP Dataflow (#2840)\n\n# Change Summary\n\nAdds a Linux host metrics receiver for OTAP Dataflow that collects\nhost-level `system.*` metrics from procfs/sysfs and emits OTAP Arrow\nmetrics directly.\n\n  Highlights:\n\n- Implements the Phase 1 host metric families with current semconv\ncoverage: CPU, memory, paging/swap, system uptime, disk, filesystem,\nnetwork, and aggregate process summary.\n- Emits metrics using OpenTelemetry semantic conventions pinned to\nschema `1.41.0`.\n- Centralizes semconv metric/attribute constants and adds a semconv\ndrift check.\n- Builds OTAP Arrow records directly without constructing intermediate\nOTLP/protobuf metric objects.\n- Uses OTAP metric rows efficiently by grouping device/interface\ndatapoints under shared metric handles.\n  - Supports per-family intervals inside one singleton receiver.\n- Enforces the one-core host collection model and duplicate receiver\nlease guard.\n- Supports host root views for container/DaemonSet deployments,\nincluding host network namespace handling via `/proc/1/net/dev`.\n- Keeps `system.process.count` limited to registered `process.state`\nvalues; Linux `procs_blocked` is intentionally not emitted until semconv\ndefines a matching state.\n\n  Notes:\n\n- `load` is intentionally not emitted in this PR because current\nOpenTelemetry system semantic conventions do not define a stable system\nload metric.\n- Receiver self-observability is implemented with the current\n`MetricSet` support. Per-family / error-class labelled internal\ntelemetry is a follow-up because the internal telemetry API does not\ncurrently support attributes on individual metric observations.\n\n## What issue does this PR close?\n\n* Closes #2741\n\n## How are these changes tested?\n\n  - Rust unit/config/projection tests\n- Semconv drift check against OpenTelemetry semantic-conventions\n`v1.41.0`\n  - df_engine runtime validation on Ubuntu\n- End-to-end validation: host metrics receiver → OTLP exporter →\nOpenTelemetry Collector → Prometheus → Grafana\n\n## Are there any user-facing changes?\n\nYes, a receiver.\n\n<img width=\"1507\" height=\"799\" alt=\"Screenshot 2026-05-04 at 9 48 05 PM\"\nsrc=\"https://github.com/user-attachments/assets/e276b067-3b14-4fbc-bfb7-31105fbb05cc\"\n/>\n<img width=\"1489\" height=\"674\" alt=\"Screenshot 2026-05-04 at 9 48 20 PM\"\nsrc=\"https://github.com/user-attachments/assets/0ddbf5e1-f7b2-4618-b4f9-7dad8d8c091d\"\n/>\n\n---------\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2026-05-07T22:16:18Z",
          "tree_id": "5b91871d3e2f637c3dd83f19cf50633f74b947a0",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/3c03f86894725a9157cb11ff00d9234df31d185a"
        },
        "date": 1778206469954,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.1299434900283813,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 5.77782078567993,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 6.260119611650486,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.32265625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.52734375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 6041.2560511554875,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 6109.518831394533,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003416,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 214347.73630119002,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 177271.56222625676,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "33842784+JakeDern@users.noreply.github.com",
            "name": "Jake Dern",
            "username": "JakeDern"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "b51f90b311efeb515a17ea8bdb2b448401f87d45",
          "message": "feat(comparison_dashboard): DFE OTAP baseline suites (#2889)\n\n# Change Summary\n\nThis PR adds the baseline suites for DFE OTAP -> OTAP.\n\nI also made small tweaks to the gitignore and the subtitle for the site.\n\n## What issue does this PR close?\n\n* Closes #2868\n* Closes #2873 \n\n## How are these changes tested?\n\nRan the suites locally and verified they were able to publish:\n\n<img width=\"1561\" height=\"184\" alt=\"image\"\nsrc=\"https://github.com/user-attachments/assets/7b632921-1c5f-4901-89c2-f89b4c568393\"\n/>\n\n\n## Are there any user-facing changes?\n\nNo.",
          "timestamp": "2026-05-07T23:01:12Z",
          "tree_id": "0104e82d215415235f4842fe92087d1df363e6c2",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/b51f90b311efeb515a17ea8bdb2b448401f87d45"
        },
        "date": 1778223969450,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.9900990128517151,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 5.770434948020994,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 6.632555684904928,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.893098958333333,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.8359375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 6032.723706323962,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 6092.453644010338,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003411,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 214143.34502546888,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 177272.08169698258,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "pritishnahar@gmail.com",
            "name": "Pritish Nahar",
            "username": "pritishnahar95"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "99b02d6115ecd7cea6b1a59fc55711aaf8a0efe0",
          "message": "fix(admin): apply OTel→Prometheus name & unit suffix rules (#2748) (#2900)\n\n# Change Summary\n\nApply the OpenTelemetry → Prometheus name & unit suffix rules in the\nadmin\nHTTP server's Prometheus text exposition.\n\nPer the OpenTelemetry spec for Prometheus exposition:\n\n- Counter metric names must end in `_total`.\n- Unit suffix derived from UCUM (e.g. `By` → `_bytes`, `By/s` →\n  `_bytes_per_second`) is inserted between the base name and `_total`.\n- A base name that already ends in `_total` must not push the unit\nsuffix\n  after it (`errors_total` + `By` → `errors_bytes_total`, not\n  `errors_total_bytes_total`).\n\nThis PR introduces `build_prom_metric_name` and the supporting UCUM\nlookup\ntables/helpers, and routes both `format_prometheus_text` and\n`agg_prometheus_text` through it. `sanitize_prom_metric_name` also now\ncollapses consecutive `_` per spec §Metric Names.\n\nThis PR is the first of two carved out of the original PR #2748. It is\nintentionally focused on metric-name and unit-suffix rules so reviewers\ncan\nevaluate that surface in isolation. Scope-label rename (`set=` →\n`otel_scope_name=`), `target_info` rendering and caching, and label-key\ncollision merging are split into a follow-up PR.\n\n## What issue does this PR close?\n\nPartially addresses #2748. The remainder (scope label, `target_info`,\nlabel\ncollision handling) will be addressed in a follow-up PR.\n\n* Refs #2748\n\n## How are these changes tested?\n\n- 11 new unit tests covering:\n  - `build_prom_metric_name` for counters with units, gauges with units,\n`_total` suffix preservation, the `<base>_<unit>_total` ordering fix,\n    and the `subtotal` (not a real `_total` suffix) edge case.\n  - `has_total_suffix` case-insensitivity.\n  - `ucum_to_prometheus_unit` for simple units, bracketed annotations\n(`{packet}/s`), compound rate units (`By/s`, `KiBy/s`, `m/s`), and the\n    intentional `By/m` rejection (UCUM `m` is meters, not minutes).\n  - `strip_curly_braces` including unbalanced-brace handling.\n  - `sanitize_prom_metric_name` underscore collapsing.\n- The existing `test_agg_prometheus_mmsc_metrics` was updated to assert\nthe\n  new `_milliseconds` unit suffix on its sub-metrics, exercising the new\n  path through `agg_prometheus_text`.\n- Full crate test suite: 33/33 telemetry lib tests pass.\n- `cargo fmt --all -- --check` clean.\n- `cargo clippy -p otap-df-admin --all-targets --all-features` clean.\n\n## Are there any user-facing changes?\n\nYes — the Prometheus text exposed at `/metrics` and\n`/metrics/aggregate` changes shape:\n\n- Counters now end in `_total` (and only one `_total`, even when the\n  source metric name already ended in `_total`).\n- Metric names gain a unit suffix derived from the metric's declared\nUCUM\n  unit, e.g. a metric named `request_duration` with unit `ms` is now\n  exposed as `request_duration_milliseconds` (or\n  `request_duration_milliseconds_total` for counters). MMSC sub-metrics\n  pick up the unit suffix as well, so `request_duration_min` becomes\n  `request_duration_milliseconds_min`.\n- Metrics whose unit is empty or `1` (dimensionless) are unchanged.\n\nDownstream Prometheus scrape consumers that hard-coded the previous\nunit-less metric names will need to update their queries. This is the\nspec-compliant naming and aligns the admin endpoint with what other\nOpenTelemetry → Prometheus exporters produce.",
          "timestamp": "2026-05-08T00:04:08Z",
          "tree_id": "14ac0aaed759e8dff4c13978297b961f45d5bfeb",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/99b02d6115ecd7cea6b1a59fc55711aaf8a0efe0"
        },
        "date": 1778227211649,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1112.0689697265625,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 5.851233195048831,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 6.456565281324974,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.460026041666666,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 494.8863686169516,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5998.364088581327,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.005694,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 213900.33036078254,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 176136.65815211998,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
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
          "id": "0b64e22da78236ed44821103b50d00075f29c55a",
          "message": "Internal logging safety: protocol buffer size limits (#2792)\n\n# Change Summary\n\nMakes protocol buffer generation limited. Use a stack-allocated array\n(256 bytes) for logged attributes, a fixed constant.\n\n~Replaces `try_into` for OtapPayloads with `try_into_with_options` and\n`try_into_with_default` to apply default settings.~\nMoved to #2811.\n\nThe default size is large to avoid surprises (256MiB, the hard max\nimplied by 4-byte placeholders). In a future PR we can tighten this and\nadd configurable limits that apply by default anywhere OTLP protos are\ngenerated in a pipeline.\n\nImplements `dropped_attributes` accounting for records the internal\nlogging pipeline.\n\nImplements string truncation for string-valued fields, with `[...]`\ninserted if possible. Truncated fields are counted in\ndropped_attributes. This applies a measure of fairness to the fields of\na log record as follows: when the string value is inserted it is allowed\nto use up to 50% of the remaining bytes or be truncated.\n\nMakes the `ProtoBuffer` padding width be determined by the available\nspace, removes a macro used to set padding width.\n\n## What issue does this PR close?\n\nFixes #1746 \nPart of #2725\n\n## How are these changes tested?\n\nA new benchmark was added, a heavy self-tracing workload with 10 string\nattributes and body, deliberately overflowing the 256B inline budget.\nThe old code would re-allocate a Vec to log this event, the new code\ndoes not.\n\n| Benchmark                 | upstream/main | jmacd/logs_sdk |       Δ |\n| ------------------------- | ------------: | -------------: | ------: |\n| `new_record`              |      170.4 µs |       183.2 µs |  +7.5%  |\n| `encode_proto`            |      195.3 µs |       195.2 µs |  ~0%    |\n| `encode_proto_with_scope` |      206.0 µs |       225.6 µs |  +9.5%  |\n| `encode_and_format`       |      862.1 µs |       559.6 µs | −35.1%  |\n| `format_with_entity`      |      934.3 µs |       627.5 µs | −32.8%  |\n\nThere is some cost for protocol writers, this is the cost of safety. For\nconsole writers, this is a win because we use a stack-allocated buffer\nfor the attributes encoding. Note that `raw_error!` is now\nallocation-free.\n\n## Are there any user-facing changes?\n\nYes. We begin to observe dropped logs attributes instead of permitting\nvery large log records in the internal logs pipeline.\n\nFor the normal data path, e.g., OTAP to OTLP conversions, the encoding\nis all-or-none. We will see encode failures instead of exceeding the\n256MiB limit.\n\n## Open questions?\n\nFor the internal logging path, a call to `with_max_remaining()` ensures\nthat each logged field can only consume half the remaining buffer space.\nThis only applies to the internal logging path. The normal OTLP logging\npath would greedily consume the whole buffer, thought at this time we do\nnot set a limit below the maximum so this will be unlikely except for\ninternal logging events. This could be refined to be more restrictive,\ne.g., to say log attribute values cannot exceed 1024 bytes as opposed to\nhalf the remaining space. Another policy would be greedier. Either way,\nthis can be refined by adding to ConversionOptions.\n\n---------\n\nCo-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-05-08T00:48:37Z",
          "tree_id": "8bbcb7f94b4004a2bdf95b0dd2cce2498d9a7b16",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/0b64e22da78236ed44821103b50d00075f29c55a"
        },
        "date": 1778232316987,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1091.5255126953125,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 5.796030099619444,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 6.607407407407408,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.687630208333335,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.16796875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 503.4189089928335,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5998.36428850783,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.005692,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 213116.39330982714,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 175297.52377197178,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
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
          "id": "c19ac244342be4170cc88260f79b7b671d3a717d",
          "message": "Avoid OtapPdata::num_items in batch_processor (#2885)\n\n# Change Summary\n\nFixes #2882. Uses the sizer's natural weight which is O(1) for batch\nprocessor bookkeeping. OTLP batching pays a penalty otherwise, due to an\n(not strictly necessary) allocation in addition to the traversal.\n\n---------\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2026-05-08T01:00:17Z",
          "tree_id": "ab810a769a16fc22e1c24da915ebd98f11da2847",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/c19ac244342be4170cc88260f79b7b671d3a717d"
        },
        "date": 1778233374152,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1110.1695556640625,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 5.697643312128508,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 6.134216308215999,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.126822916666665,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.45703125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 503.4381720661277,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 6092.455167037546,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003396,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 212002.07261075356,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 174145.47213472487,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
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
          "id": "db744cd7747e889fa65e03bbd49597b95b688e45",
          "message": "Wait for result stream concurrency (#2848)\n\n# Change Summary\n\nThis PR improves OTAP wait-for-result throughput by allowing the OTAP\nreceiver to process multiple in-flight requests per incoming stream,\nbounded by a new per-stream concurrency limit. It also makes OTAP\nexporter response handling more robust by correlating responses by\n`batch_id`, correctly handling out-of-order statuses and non-OK batch\nresponses.\n\nThe change also fixes rejection metrics so concurrency-limit rejections\nare counted separately from memory-pressure rejections.\n\n## What issue does this PR close?\n\nFixes #1325\n\n## How are these changes tested?\n\nJake tested this new branch with a throughput of 100K logs/s, and the\nresults are the same with or without `wait_for_result`.\n\n* Added coverage for out-of-order OTAP exporter `BatchStatus` handling.\n* Added coverage for non-OK batch statuses producing failed\nexports/NACKs.\n* Ran `cargo xtask check` successfully before merging latest\n`origin/main`.\n\n## Are there any user-facing changes?\n\nYes.\n\n* OTAP receiver has a new optional `max_concurrent_requests_per_stream`\nsetting.\n* The default wait-for-result behavior can now process multiple\nin-flight messages on a single stream.\n* Concurrency-limit rejections no longer inflate memory-pressure\nrejection metrics.\n\n---------\n\nCo-authored-by: Joshua MacDonald <jmacd@users.noreply.github.com>\nCo-authored-by: Utkarsh Umesan Pillai <66651184+utpilla@users.noreply.github.com>",
          "timestamp": "2026-05-08T02:42:43Z",
          "tree_id": "480982646287a8b4cfcd25e40f4ac98cb0392edc",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/db744cd7747e889fa65e03bbd49597b95b688e45"
        },
        "date": 1778242744541,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1111.8643798828125,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 5.819647159181985,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 6.267276096355776,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.766145833333333,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.24609375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 503.4378700205015,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 6100.984357028112,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003432,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 215802.34840211214,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 178049.78799734323,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "33842784+JakeDern@users.noreply.github.com",
            "name": "Jake Dern",
            "username": "JakeDern"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "09787d85f092f284a0cf74e00e55e51cdda76d54",
          "message": "feat(comparison_dashboard): DFE OTLP HTTP baseline suites (#2895)\n\n# Change Summary\n\nThis PR adds the DFE OTLP baseline suites + associated templates.\n\n## What issue does this PR close?\n\n* Closes #2878 \n* Closes #2894\n\n## How are these changes tested?\n\n## Are there any user-facing changes?\n\nNo.",
          "timestamp": "2026-05-08T03:30:53Z",
          "tree_id": "7702c43c244f4a7bcd8cea1b6441870f4753c80c",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/09787d85f092f284a0cf74e00e55e51cdda76d54"
        },
        "date": 1778243782375,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1111.8643798828125,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 5.767537776570308,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 6.2977385924207265,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.46640625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.30078125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 503.4343881318143,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 6100.942161258427,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003847,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 214356.6381272945,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 176587.82705786818,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "33842784+JakeDern@users.noreply.github.com",
            "name": "Jake Dern",
            "username": "JakeDern"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "b7533299e0f977fd19733c1b5edd292c4ceb4a21",
          "message": "feat(comparison_dashboard): DFE OTLP baseline templates (#2893)\n\n# Change Summary\n\nThis PR adds the DFE OTLP baseline suites + associated templates.\n\n## What issue does this PR close?\n\n* Closes #2874 \n* Closes #2875\n\n## How are these changes tested?\n\n\n\n## Are there any user-facing changes?\n\n <!-- If yes, provide further info below -->",
          "timestamp": "2026-05-08T03:33:53Z",
          "tree_id": "74b5486dfcfa3198519dbec04b8be3e092fb0882",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/b7533299e0f977fd19733c1b5edd292c4ceb4a21"
        },
        "date": 1778244826555,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1113.559326171875,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 5.8209808045938,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 6.169309180227203,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.136197916666667,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.49609375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 503.4374672935636,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 6109.512314952399,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00348,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 215188.19754203793,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 177362.12760449055,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "pritishnahar@gmail.com",
            "name": "Pritish Nahar",
            "username": "pritishnahar95"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "3f3787a061b54e7679aeceb4f2d58407bce2ff79",
          "message": "fix(admin): add target_info, scope label, and label-collision merging (#2748) (#2904)\n\nfix(admin): add target_info, scope label, and label-collision merging\n(#2748)\n\nPer the OpenTelemetry spec for Prometheus exposition:\n- Replace the ad-hoc `set=\"<scope>\"` label with `otel_scope_name`.\n`otel_scope_version` is omitted when empty (MetricsDescriptor does not\nyet carry a version).\n- Emit a `target_info` gauge derived from resource attributes.\nPre-render once at server startup and cache as `Arc<str>` on AppState so\nthe hot path is a single `push_str`. Empty attribute map yields an empty\nblock (which the spec mandates).\n- Merge label values whose original keys collide after sanitization\n(joined with `;`) and collapse consecutive `_` in sanitized label keys\nper spec §Metric Attributes.\n- Extract per-metric emission into `emit_scalar_metric`,\n`emit_mmsc_metric`, and `emit_sample_line` helpers; add `# UNIT`\nmetadata lines for typed metrics. Histogram→gauge fallback is documented\nat the call site (proper histogram family requires buckets/sum/count\nwhich the registry doesn't store yet).\n\nThe controller passes `HashMap::new()` for now; wiring real resource\nattributes is tracked under `TODO(#2748)` and follows once the\ncontroller exposes them.\n\n# Change Summary\n\nThis is **PR-2 of 2** splitting the original Prometheus text-formatter\nOTel-spec compliance work (#2748). PR-1 (#2900, merged) handled metric\nname and unit suffix rules. This PR completes the spec compliance by\naddressing scope identification, resource identity (`target_info`), and\nlabel-key sanitization edge cases.\n\nHighlights:\n- **Scope label**: emits `otel_scope_name=\"<scope>\"` instead of the\nad-hoc `set=\"<scope>\"` so downstream Prometheus consumers can identify\nthe originating instrumentation scope per the OTel/Prometheus interop\nspec.\n- **`target_info` gauge**: rendered once at admin server startup from\nthe supplied resource attribute map and cached as an `Arc<str>` on\n`AppState`. Each scrape pays only a single `push_str` — no per-scrape\nallocation, no formatting, no locking.\n- **Label sanitization & collision merging**: keys like `http.method`\nand `http_method` both sanitize to `http_method`; their values are\njoined with `;` rather than silently overwriting one another.\nConsecutive underscores in sanitized keys are collapsed to a single `_`,\nmatching the existing rule for metric names.\n- **Hot-path refactor**: per-metric emission split into\n`emit_scalar_metric`, `emit_mmsc_metric`, and `emit_sample_line` so each\nformatter (Prometheus / JSON / line-protocol) reuses the same shape. `#\nUNIT` metadata lines are now emitted for typed metrics.\n- **Documented fallback**: histograms still render as a gauge of the\ncount, with an inline comment at the call site explaining that a full\n`_bucket` / `_sum` / `_count` family requires bucket boundaries the\nregistry does not yet store.\n\n## What issue does this PR close?\n\n* Part of #2748 (PR-1 merged in #2900; this PR completes the remaining\nspec-compliance items).\n\n## How are these changes tested?\n\n- 45/45 `otap-df-admin` lib tests pass, including:\n- `test_format_prometheus_text_e2e_otel_compliance` — end-to-end fixture\nasserting `# HELP` / `# TYPE` / `# UNIT` ordering, `otel_scope_name`\nplacement, `target_info` block, and `_total` suffix on counters.\n- `test_sanitize_and_merge_label_pairs_collisions_use_semicolon` —\nverifies `;`-joined merging of values whose original keys collide after\nsanitization.\n- `test_sanitize_and_merge_label_pairs_distinct_keys_unchanged` — guards\nagainst false-positive merging.\n- `test_sanitize_prom_label_key_collapses_underscores` — verifies the\n`_+` → `_` rule on label keys.\n- `cargo fmt -p otap-df-admin --check` clean.\n- `cargo clippy -p otap-df-admin --all-targets -- -D warnings` clean.\n- Manually scraped a running admin server against a Prometheus 2.x\ninstance to confirm the output parses without warnings and `target_info`\njoins correctly via `* on (job, instance) group_left(...) target_info`.\n\n## Are there any user-facing changes?\n\nYes — the `/metrics` endpoint output changes in three ways visible to\nscrapers:\n\n1. The label `set=\"<scope>\"` is replaced by `otel_scope_name=\"<scope>\"`.\nDashboards and alerts that grouped by `set` must be updated.\n2. A new `target_info` gauge is emitted (empty block when no resource\nattributes are configured, which is the current default — the controller\nstill passes `HashMap::new()` pending follow-up).\n3. Label keys that previously collided silently are now merged with `;`\nseparators rather than one value overwriting the other. This is a\ncorrectness fix; the previous behavior was non-deterministic.\n\nNo configuration changes are required. The change is opt-in only in the\nsense that it affects output of the existing admin Prometheus endpoint\nthat already had to be enabled to be scraped.",
          "timestamp": "2026-05-08T15:42:59Z",
          "tree_id": "a31030f95fbbdb8466a6f8dffeac343c9df6b7e8",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/3f3787a061b54e7679aeceb4f2d58407bce2ff79"
        },
        "date": 1778268679186,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1100,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 5.793517244252253,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 6.528780186713988,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.561588541666666,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.88671875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 503.44544644109914,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 6041.345357293189,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002529,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 215433.40075827594,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 176642.6101172061,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "pritishnahar@gmail.com",
            "name": "Pritish Nahar",
            "username": "pritishnahar95"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "721dff5d2d979d1d98d3bc333f8e47887a608fa3",
          "message": "feat(admin): wire engine resource attributes into target_info (#2748) (#2907)\n\nfeat(admin): wire engine resource attributes into target_info (#2748)\n\nReplace the `HashMap::new()` placeholder in the controller with the\nengine's `telemetry.resource` map so `target_info` carries real resource\nmetadata.\n\nPush the typed `AttributeValue` enum down to `render_target_info`\ninstead of pre-flattening to strings at the API boundary.\nStringification now happens once at startup, in the only function that\nneeds the flat form, via a new `AttributeValue::to_string_value()` on\nthe config crate.\n\nConversion is round-trip lossless: scalars use Display (Ryu for f64),\narrays render as bare JSON arrays of the inner Vec (preserving element\ntype and order, avoiding the externally-tagged enum form).\n\nHot scrape path is unchanged: `target_info` is still pre-rendered once\nand cached as `Arc<str>`.\n\nAdds tests for scalar formatting, f64 bit-equal round-trip across edge\ncases (PI, 1e±300, EPSILON, -0.0), bare-JSON array encoding, and an\nend-to-end `render_target_info` test exercising every variant including\nescaped JSON arrays in Prometheus label values.\n\nCloses the `TODO(#2748)` for resource attribute wiring.\n\n# Change Summary\n\nThis is the **resource-attribute follow-up to #2748**, building on the\nmerged PR-1 (#2900 — name/unit suffix rules) and PR-2 (#2904 — scope\nlabel, `target_info` block, label-collision merging). PR-2 introduced\nthe `target_info` gauge but the controller was passing `HashMap::new()`\nwith a `TODO(#2748)` because the resource attributes had not yet been\nthreaded through. This PR closes that TODO.\n\nThe change has two parts:\n\n1. **Wiring (1 line of behavior change in the controller):** the admin\nserver is now spawned with\n`engine_config.engine.telemetry.resource.clone()` instead of the empty\nplaceholder, so any `engine.telemetry.resource` keys declared in YAML\nnow appear as `target_info` labels on every Prometheus scrape.\n\n2. **Type-correctness refactor at the admin API boundary:** previously,\n`pub async fn run(...)` accepted `HashMap<String, String>`, forcing\nevery caller to pre-flatten typed values. The signature now accepts\n`HashMap<String, AttributeValue>` (the same enum already defined in\n`otap_df_config::pipeline::telemetry`). Stringification is moved down\ninto `render_target_info` — the only function that actually needs the\nflat form. This eliminates a parallel string-only type at the boundary,\nmatches the pattern already used elsewhere in the codebase, and means\nfuture callers can pass typed values without first lossy-flattening\nthem.\n\nThe new helper `AttributeValue::to_string_value()` is the single source\nof truth for the typed → string conversion. It is round-trip lossless\nfor every variant:\n\n* `String` — cloned verbatim (no behavioral change vs. the prior code).\n* `Bool` — `\"true\"` / `\"false\"`.\n* `I64` — decimal `Display`.\n* `F64` — Rust's `Display`, which uses **Ryu** under the hood: the\nshortest decimal representation that round-trips back to a bit-equal\n`f64`. Verified by a dedicated test against `PI`, `1e±300`, `EPSILON`,\nand `-0.0`.\n* `Array` — encoded as a **bare JSON array** of the inner `Vec` (e.g.\n`[\"edge\",\"us-west\"]`, `[1,2,3]`, `[true,false]`). The inner `Vec` is\nserialized directly so the output is not the externally-tagged\n`{\"String\":[...]}` form that serializing the enum variant would yield.\nGeneric JSON tooling, OTel SDKs, and Prometheus consumers all parse the\nresult without special-casing.\n\nThe hot scrape path is unchanged: `render_target_info` is still called\nexactly once, at admin server startup, and the result is cached as\n`Arc<str>` on `AppState` and `push_str`'d on every scrape. No per-scrape\nallocation, no per-scrape conversion, no per-scrape locking.\n\n## What issue does this PR close?\n\n* Closes #2748 \n\n## How are these changes tested?\n\n* New unit tests on `AttributeValue::to_string_value()` in\n`otap-df-config`.\n* New end-to-end test on the renderer in `otap-df-admin`\n  (`test_format_prometheus_text_e2e_otel_compliance` extension).\n* New regression test for collision-merge ordering in `otap-df-admin`\n\n(`test_sanitize_and_merge_label_pairs_collision_order_is_lexicographic`),\n  covering multiple `Vec` permutations and a `HashMap` input to verify\n  the lex-by-original-key ordering required by the OTel→Prometheus spec.\n* Comment update on existing\n`test_sanitize_and_merge_label_pairs_collisions_use_semicolon`.\n\n* Full results:\n  * `cargo test -p otap-df-admin --lib` → 49 passed (47 prior + 2 new).\n  * `cargo test -p otap-df-config --lib` → all green, 3 new tests added.\n* `cargo clippy -p otap-df-admin --all-targets --no-deps -- -D warnings`\n→ clean.\n  * `cargo build -p otap-df-{admin,controller,config}` → clean.\n## Are there any user-facing changes?\n\nYes — but additive only. Resource attributes declared under\n`engine.telemetry.resource` in the YAML config now flow into the\n`target_info` gauge labels at `/metrics`. Example:\n\n```yaml\nengine:\n  telemetry:\n    resource:\n      service.name: \"edge-collector\"\n      service.instance.port: 4317\n      deployment.canary: true\n      service.tags: [\"edge\", \"us-west\"]\n```\n\n…now produces (label order is unspecified):\n\n```\n# HELP target_info Target metadata\n# TYPE target_info gauge\ntarget_info{service_name=\"edge-collector\",service_instance_port=\"4317\",deployment_canary=\"true\",service_tags=\"[\\\"edge\\\",\\\"us-west\\\"]\"} 1\n```\n\nPreviously, the `target_info` block was always empty, so any consumer\nthat was relying on its emptiness will see real labels for the first\ntime. No config schema change — the `engine.telemetry.resource` field\nalready existed and was already being consumed by the OTel SDK metrics\npath; this PR just stops dropping it on the Prometheus path.",
          "timestamp": "2026-05-08T18:54:36Z",
          "tree_id": "68dd5f1b48e2ec936d9e4768afd2bdaf8e126d05",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/721dff5d2d979d1d98d3bc333f8e47887a608fa3"
        },
        "date": 1778273929839,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1111.8643798828125,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 5.8006536138738705,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 6.308585546633724,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.021484375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.28125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 503.4087830467627,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 6100.631862346361,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.006899,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 215180.34670252778,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 176357.52226909538,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "213113461+gyanranjanpanda@users.noreply.github.com",
            "name": "Gyan ranjan",
            "username": "gyanranjanpanda"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "0f3ffaf2a96c0a96937a6c92ce34a7f54d1fffd6",
          "message": "feat(query-engine): implement log10 scalar function (#2862)\n\nCloses #2832\n\n# Implementing `log` Scalar Function\n\n## Objective\nAdd support for the `log()` mathematical scalar function in the OPL/OTAP\nquery engine.\n\n## Changes Made\n- Added `math_expressions` feature to `datafusion` dependency in\n`rust/otap-dataflow/crates/query-engine/Cargo.toml`.\n- Registered the `log` function name in\n`rust/otap-dataflow/crates/query-engine/src/consts.rs`.\n- Added the `log` external function to the parser configuration in\n`rust/otap-dataflow/crates/query-engine/src/parser.rs`.\n- Mapped the `log` function to the corresponding DataFusion UDF in\n`rust/otap-dataflow/crates/query-engine/src/pipeline/expr.rs`.\n- Ensured arguments to `log` are coerced to `Float64` in `expr.rs` to\ncorrectly interface with DataFusion's scalar UDF logic.\n- Added comprehensive unit tests in\n`rust/otap-dataflow/crates/query-engine/src/pipeline/assign.rs` to\nverify that the `log` function computes correct values and works\nseamlessly with both KQL and OPL parsers.\n\nSigned-off-by: Gyan Ranjan Panda <sanupanda141@gmail.com>\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-05-08T19:18:41Z",
          "tree_id": "68fd56739e3d7ad1bbd853ad5512f9000409dad7",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/0f3ffaf2a96c0a96937a6c92ce34a7f54d1fffd6"
        },
        "date": 1778309894317,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1110.1695556640625,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 5.728374331517571,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 6.167832906281544,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.047005208333335,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.109375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 503.437844850049,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 6092.451207168389,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003435,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 214076.54051723715,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 175352.89814781252,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
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
          "id": "8401d51bc2b808d6cace3d52e52bafa49db811c7",
          "message": "Rename engine.metrics and pipeline.metrics meters (#2888)\n\nDrops the redundant trailing `.metrics` from the engine-wide and\nper-pipeline meter/scope names:\n\n- `engine.metrics` → `engine`\n- `pipeline.metrics` → `pipeline`\n\nA meter name already names a set of metrics, so the trailing `.metrics`\nwas tautological in scrape output and view selectors. Instrument names\nare unchanged — only the scope/meter names.\n\nPart of #2531. Follow-up to #2879 (which renamed `otlp.receiver.metrics`\n→ `otlp.receiver`). The remaining per-component renames (core-nodes\nreceivers/processors/exporters, contrib-nodes, validation, docs sweep)\nwill land as separate smaller PRs.\n\n## Breaking change for downstream consumers\n\nAnything that selects metrics by **scope/meter name** must be updated.\nThe instrument names are unchanged — only the scope name changes.\n\nExamples that need updating:\n\n- View configurations that filter by `ScopeName` (e.g. `ScopeName:\n\"engine.metrics\"` → `ScopeName: \"engine\"`, `ScopeName:\n\"pipeline.metrics\"` → `ScopeName: \"pipeline\"`).\n- Prometheus relabeling/alerting that keys off the `set=\"…\"` label.\n- Dashboards or queries that group by OTLP scope name.\n\nEffective emitted name mapping (`<scope>.<instrument>`):\n\n| Before | After |\n|---|---|\n| `engine.metrics.memory_rss` | `engine.memory_rss` |\n| `engine.metrics.cpu_utilization` | `engine.cpu_utilization` |\n| `pipeline.metrics.uptime` | `pipeline.uptime` |\n| `pipeline.metrics.memory_usage` | `pipeline.memory_usage` |\n| `pipeline.metrics.cpu_utilization` | `pipeline.cpu_utilization` |\n| (…and the rest of the `pipeline.*` set) | (analogous) |\n\n## In-repo consumers updated in this PR\n\n- Rust: `enginectl` TUI (`refresh.rs`, `tests.rs`), `admin-types`\nwire-shape roundtrip tests\n- Admin UI JS: `engine-metrics.js`, `main.js`, `charts-controller.js`,\n`selectors-ui.js`, `polling-controller.test.mjs`\n- Python: `scripts/engine-metrics.py`\n- Docs: `engine/telemetry.md`, `docs/admin/architecture.md`,\n`docs/memory-limiter-phase1.md`, `docs/telemetry/metrics-guide.md`",
          "timestamp": "2026-05-08T23:41:58Z",
          "tree_id": "4e6bcc922198d52619fafa98434b46a51ed3610e",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/8401d51bc2b808d6cace3d52e52bafa49db811c7"
        },
        "date": 1778311944949,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1093.2203369140625,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 5.775758885043596,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 6.425740583185088,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.811848958333332,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.796875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 503.2782644483704,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 6005.2186130788605,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.022461,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 214031.23229471088,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 175360.22738282548,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "wbutler@microsoft.com",
            "name": "Will Butler",
            "username": "wbutler"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "33569b3c088ffa0fb72f32f5f5007547c00ed460",
          "message": "Fix timeouts in flaky test_no_partitioning UT (#2906)\n\nWorking on a previous merge, I got caught up in a merge validation\nfailure on `test_no_partitioning` in `parquet_exporter`.\n\nLooking at the source, the 1s timeout is defined in `mod.rs` at:\n\n```rust\n        test_runtime\n            .set_exporter(exporter)\n            .run_test(logs_scenario(\n                num_rows,\n                Instant::now().add(Duration::from_secs(1)),\n                Duration::from_secs(1),\n            ))\n```\n\nThe result gets fed into the async block within the `logs_scenario`\nhelper func as `shutdown_timeout`:\n\n```rust\n    fn logs_scenario(\n        num_rows: usize,\n        shutdown_timeout: Instant,\n    ) -> impl FnOnce(TestContext<OtapPdata>) -> Pin<Box<dyn Future<Output = ()>>> {\n        move |ctx| {\n            Box::pin(async move {\n                let mut consumer = Consumer::default();\n                let otap_batch = consumer\n                    .consume_bar(&mut fixtures::create_simple_logs_arrow_record_batches(\n                        SimpleDataGenOptions {\n                            num_rows,\n                            ..Default::default()\n                        },\n                    ))\n                    .unwrap();\n\n                ctx.send_pdata(OtapPdata::new_default(\n                    OtapArrowRecords::Logs(from_record_messages(otap_batch).unwrap()).into(),\n                ))\n                .await\n                .expect(\"Failed to send  logs message\");\n\n                ctx.send_shutdown(shutdown_timeout, \"test completed\")\n                    .await\n                    .unwrap();\n            })\n        }\n    }\n```\n\nBut because the `Instant` is determined at the call time of\n`logs_scenario`, the shutdown timeout clock is already running during\nsetting up the exporter and running the actual test.\n\n## Changes\n\nThe fix is to pass in a `Duration` and then properly start the clock\nwhen we call `send_shutdown` by converting to an `Instant` at that time.\nThis is consistent with the pattern in other UT's, in the module, like\n`test_adaptive_schema_dict_upgrade_write`, `test_metrics`,\n`test_adaptive_schema_optional_columns`, and others, which all compute\nthe `Instant` at the call time of `send_shutdown`.\n\nThis fix also addresses `test_with_partitioning`, which uses the same\nhelper and the same anti-pattern.\n\n## Validation\n\nThe following commands run clean:\n\n`cargo check --workspace`\n`cargo test -p otap-df-core-nodes -- parquet_exporter`\n`cargo clippy -p otap-df-core-nodes --all-targets -- -D warnings`\n`cargo fmt --all -- --check`\n`cargo xtask quick-check`\n\n## Notes\n\n`cargo check -p otap-df-core-nodes` fails on trying to resolve `ring` in\n`crypto`. I'm not going after that as non-germane to this fix.",
          "timestamp": "2026-05-09T00:53:15Z",
          "tree_id": "dba956b705c4924570bd04af7855b594bebbdeeb",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/33569b3c088ffa0fb72f32f5f5007547c00ed460"
        },
        "date": 1778326168235,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1093.2203369140625,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 5.796087327378047,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 6.36772027539259,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.691276041666665,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.9765625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 503.41055317034,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 6006.7971090155825,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.006688,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 216267.93129714325,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 177662.08235516792,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
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
          "id": "3350e33e1224477600597d5a141b16550f091aa3",
          "message": "Rename core-nodes receiver meters to drop redundant .metrics suffix (#2912)\n\nDrops the redundant trailing `.metrics` from core-nodes receiver\nmeter/scope names:\n\n- `traffic_generator.receiver.metrics` → `traffic_generator.receiver`\n- `topic.receiver.metrics` → `topic.receiver`\n- `otap.receiver.metrics` → `otap.receiver`\n- `syslog_cef.receiver.metrics` → `syslog_cef.receiver`\n\nA meter name already names a set of metrics, so the trailing `.metrics`\nwas tautological in scrape output and view selectors. Instrument names\nare unchanged — only the scope/meter names.\n\nPart of #2531. Follow-up to #2879 (`otlp.receiver`) and #2888 (`engine`,\n`pipeline`). The remaining per-component renames (core-nodes\nprocessors/exporters, contrib-nodes, validation, docs sweep) will land\nas separate smaller PRs.\n\n## Breaking change for downstream consumers\n\nAnything that selects metrics by **scope/meter name** must be updated.\nInstrument names are unchanged.\n\nExamples that need updating:\n\n- View configurations that filter by `ScopeName` (e.g. `ScopeName:\n\"traffic_generator.receiver.metrics\"` → `ScopeName:\n\"traffic_generator.receiver\"`).\n- Prometheus relabeling/alerting that keys off the `set=\"…\"` label.\n- Dashboards or queries that group by OTLP scope name.\n\nEffective emitted name mapping (`<scope>.<instrument>`), examples:\n\n| Before | After |\n|---|---|\n| `traffic_generator.receiver.metrics.logs.produced` |\n`traffic_generator.receiver.logs.produced` |\n| `topic.receiver.metrics.forwarded_messages` |\n`topic.receiver.forwarded_messages` |\n| `otap.receiver.metrics.refused_memory_pressure` |\n`otap.receiver.refused_memory_pressure` |\n| `syslog_cef.receiver.metrics.received_logs_total` |\n`syslog_cef.receiver.received_logs_total` |",
          "timestamp": "2026-05-09T04:31:13Z",
          "tree_id": "91b9ec2deaf45fac5ccc8f0620b5da822d60aef8",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/3350e33e1224477600597d5a141b16550f091aa3"
        },
        "date": 1778333955880,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1068.3333740234375,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 5.769139874268314,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 6.499030592586884,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.104557291666666,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.41015625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 511.92141153130643,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5980.948491390764,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.009211,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 212984.6530338108,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 174346.7271494869,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
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
          "id": "c3cf2beba17f14a89341af778253c77c1a0a4346",
          "message": "Fix validate-configs by renaming Jinja template otap-otap.yaml to .yaml.j2 (#2913)\n\nThe `validate-configs` CI job has been failing on PRs since #2893 landed\n(DFE OTLP baseline templates). The validator script\n(`rust/otap-dataflow/scripts/validate-configs.sh`) discovers configs by\nglobbing every `*.yaml`/`*.yml` containing `version: otel_dataflow/v1`\nand runs `df_engine --validate-and-exit` on each. The new template\n`tools/pipeline_perf_test/test_suites/comparison_dashboard/templates/engine/otap-otap.yaml`\ncontains unrendered Jinja2 placeholders (`{{core_start}}`,\n`{{core_end}}`, …), so the YAML deserializer fails:\n\n```\nError: DeserializationError {\n  format: \"YAML\",\n  details: \"policies.resources.core_allocation.set[0].start: invalid type: map, expected usize at line 12 column 18\"\n}\n```\n\nThe other Jinja templates in the same directory (`otlp-otlp.yaml.j2`,\n`otlphttp-otlphttp.yaml.j2`, `loadgen/*.yaml.j2`, `backend/*.yaml.j2`)\nalready use the `.yaml.j2` extension, which the validator skips. This PR\nbrings `otap-otap.yaml` in line with that convention by renaming it to\n`otap-otap.yaml.j2` and updating the 3 sibling suite files in\n`tools/comparison_dashboard/suites/dfe/` that reference it (the matching\n`dfe-logs-otlp-*-baseline.yaml` files already reference their template\nwith the `.j2` suffix).\n\n## Validation\n\nLocally re-running `./scripts/validate-configs.sh` after the fix:\n\n```\nConfig validation: 73/73 passed, 0 failed.\n```\n\n(Previously 1 failed, 72 passed.)",
          "timestamp": "2026-05-09T23:18:23Z",
          "tree_id": "724d637dae2c372201acc4eb91b5c93074821b95",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/c3cf2beba17f14a89341af778253c77c1a0a4346"
        },
        "date": 1778372292278,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1113.559326171875,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 5.750940552997679,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 6.185131894484412,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.653515625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.88671875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 503.43778611900296,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 6109.516184088239,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003442,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 214046.20163526517,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 175472.51757424982,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
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
          "id": "2f734a94eaaa1abff2b134e65b6c276e4f05b7cf",
          "message": "chore(deps): update rust crate sysinfo to 0.39 (#2921)\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n|---|---|---|---|\n| [sysinfo](https://redirect.github.com/GuillaumeGomez/sysinfo) |\nworkspace.dependencies | minor | `0.38` → `0.39` |\n\n---\n\n### Release Notes\n\n<details>\n<summary>GuillaumeGomez/sysinfo (sysinfo)</summary>\n\n###\n[`v0.39.1`](https://redirect.github.com/GuillaumeGomez/sysinfo/blob/HEAD/CHANGELOG.md#0391)\n\n[Compare\nSource](https://redirect.github.com/GuillaumeGomez/sysinfo/compare/v0.39.0...v0.39.1)\n\n- Linux: Fix wrong network numbers computation.\n\n###\n[`v0.39.0`](https://redirect.github.com/GuillaumeGomez/sysinfo/blob/HEAD/CHANGELOG.md#0390)\n\n[Compare\nSource](https://redirect.github.com/GuillaumeGomez/sysinfo/compare/v0.38.4...v0.39.0)\n\n- Update minimum supported rust version to `1.95`.\n- Add new `NetworkData::operational_state` API.\n- Add new `Process::cgroup_limits` API (only returning data on Linux).\n- All supported systems other than Windows: Improve performance of\n`Networks::refresh*`.\n- All supported systems other than Windows: Fix soundness issue when\nretrieving users.\n- Linux: Take into account parent cgroup memory limits.\n- Linux: Fix panic when retrieving process information on `ESXi`.\n- FreeBSD: Use the name of dataset as `name` for zfs disks.\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: (UTC)\n\n- Branch creation\n  - \"before 8am on Monday\"\n- Automerge\n  - At any time (no schedule defined)\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My4xNTkuMiIsInVwZGF0ZWRJblZlciI6IjQzLjE1OS4yIiwidGFyZ2V0QnJhbmNoIjoibWFpbiIsImxhYmVscyI6WyJkZXBlbmRlbmNpZXMiXX0=-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2026-05-11T01:51:07Z",
          "tree_id": "519f80147bda623218c625b0932de6f27ac68546",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/2f734a94eaaa1abff2b134e65b6c276e4f05b7cf"
        },
        "date": 1778468108676,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1113.559326171875,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 5.743141166029931,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 6.173442572267739,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.150130208333334,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.00390625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 503.41022599016276,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 6109.181725575534,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.006727,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 215482.8200552889,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 176848.01454013732,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
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
          "id": "5c5bb9faecc77a73c95888c1bb58724ea78de875",
          "message": "fix(deps): update module google.golang.org/grpc to v1.81.0 (#2923)\n\nThis PR contains the following updates:\n\n| Package | Change |\n[Age](https://docs.renovatebot.com/merge-confidence/) |\n[Confidence](https://docs.renovatebot.com/merge-confidence/) |\n|---|---|---|---|\n| [google.golang.org/grpc](https://redirect.github.com/grpc/grpc-go) |\n`v1.80.0` → `v1.81.0` |\n![age](https://developer.mend.io/api/mc/badges/age/go/google.golang.org%2fgrpc/v1.81.0?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/go/google.golang.org%2fgrpc/v1.80.0/v1.81.0?slim=true)\n|\n\n---\n\n### Release Notes\n\n<details>\n<summary>grpc/grpc-go (google.golang.org/grpc)</summary>\n\n###\n[`v1.81.0`](https://redirect.github.com/grpc/grpc-go/releases/tag/v1.81.0):\nRelease 1.81.0\n\n[Compare\nSource](https://redirect.github.com/grpc/grpc-go/compare/v1.80.0...v1.81.0)\n\n### Behavior Changes\n\n- balancer/rls: Switch gauge metrics to asynchronous emission (once per\ncollection cycle) to reduce telemetry noise and align with other gRPC\nlanguage implementations.\n([#&#8203;8808](https://redirect.github.com/grpc/grpc-go/issues/8808))\n\n### Dependencies\n\n- Minimum supported Go version is now 1.25.\n([#&#8203;8969](https://redirect.github.com/grpc/grpc-go/issues/8969))\n\n### Bug Fixes\n\n- xds: Use the leaf cluster's security config for the TLS handshake\ninstead of the aggregate cluster's config.\n([#&#8203;8956](https://redirect.github.com/grpc/grpc-go/issues/8956))\n- transport: Send a `RST_STREAM` when receiving an `END_STREAM` when the\nstream is not already half-closed.\n([#&#8203;8832](https://redirect.github.com/grpc/grpc-go/issues/8832))\n- xds: Fix ADS resource name validation to prevent a panic.\n([#&#8203;8970](https://redirect.github.com/grpc/grpc-go/issues/8970))\n\n### New Features\n\n- grpc/stats: Add support for custom labels in per-call metrics ([gRFC\nA108](https://redirect.github.com/grpc/proposal/blob/master/A108-otel-custom-per-call-label.md)).\n([#&#8203;9008](https://redirect.github.com/grpc/grpc-go/issues/9008))\n- xds: Add support for Server Name Indication (SNI) and SAN validation\n([gRFC\nA101](https://redirect.github.com/grpc/proposal/blob/master/A101-SNI-setting-and-SNI-SAN-validation.md)).\nDisabled by default. To enable, set `GRPC_EXPERIMENTAL_XDS_SNI=true`\nenvironment variable.\n([#&#8203;9016](https://redirect.github.com/grpc/grpc-go/issues/9016))\n- xds: Add support to control which fields get propagated from ORCA\nbackend metric reports to LRS load reports ([gRFC\nA85](https://redirect.github.com/grpc/proposal/blob/master/A85-lrs-custom-metrics-changes.md)).\nDisabled by default. To enable, set\n`GRPC_EXPERIMENTAL_XDS_ORCA_LRS_PROPAGATION=true`.\n([#&#8203;9005](https://redirect.github.com/grpc/grpc-go/issues/9005))\n- xds: Add metrics to track xDS client connectivity and cached resource\nstate ([gRFC\nA78](https://redirect.github.com/grpc/proposal/blob/master/A78-grpc-metrics-wrr-pf-xds.md)).\n([#&#8203;8807](https://redirect.github.com/grpc/grpc-go/issues/8807))\n- stats/otel: Enhance `grpc.subchannel.disconnections` metric by adding\ndisconnection reason to the `grpc.disconnect_error` label ([gRFC\nA94](https://redirect.github.com/grpc/proposal/blob/master/A94-subchannel-otel-metrics.md)).\nThis provides granular insights into why subchannels are closing.\n([#&#8203;8973](https://redirect.github.com/grpc/grpc-go/issues/8973))\n- mem: Add `mem.Buffer.Slice()` API to slice the buffer like a slice.\n([#&#8203;8977](https://redirect.github.com/grpc/grpc-go/issues/8977))\n  - Special Thanks: [@&#8203;ash2k](https://redirect.github.com/ash2k)\n\n### Performance Improvements\n\n- alts: Pool read buffers to lower memory utilization when sockets are\nunreadable.\n([#&#8203;8964](https://redirect.github.com/grpc/grpc-go/issues/8964))\n- transport: Pool HTTP/2 framer read buffers to reduce idle memory\nconsumption. Currently limited to Linux for ALTS and non-encrypted\ntransports (TCP, Unix). To disable, set\n`GRPC_GO_EXPERIMENTAL_HTTP_FRAMER_READ_BUFFER_POOLING=false` and report\nany issues.\n([#&#8203;9032](https://redirect.github.com/grpc/grpc-go/issues/9032))\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: (UTC)\n\n- Branch creation\n  - \"before 8am on Monday\"\n- Automerge\n  - At any time (no schedule defined)\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My4xNTkuMiIsInVwZGF0ZWRJblZlciI6IjQzLjE1OS4yIiwidGFyZ2V0QnJhbmNoIjoibWFpbiIsImxhYmVscyI6WyJkZXBlbmRlbmNpZXMiXX0=-->\n\n---------\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: otelbot <197425009+otelbot@users.noreply.github.com>\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2026-05-11T03:48:08Z",
          "tree_id": "4df6d22158be94ad40a81cb7e3317b6d87b80a25",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/5c5bb9faecc77a73c95888c1bb58724ea78de875"
        },
        "date": 1778474003675,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1111.8643798828125,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 5.784234837069658,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 6.058250985850151,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.438671875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.80078125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 503.34992304118265,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 6099.918558888909,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.013916,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 214991.45878161458,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 176309.38482303938,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
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
          "id": "d83298856be7ea8aca91e00a095a5861d6629389",
          "message": "Rename remaining meters to drop redundant .metrics suffix (#2917)\n\nFinal cleanup PR for the meter-rename series. Drops the redundant\ntrailing `.metrics` from every remaining meter/scope name across the\nrepo.\n\nCloses #2531. Follow-up to #2879 (`otlp.receiver`), #2888 (`engine`,\n`pipeline`), and #2912 (core-nodes receivers).\n\nA meter name already names a set of metrics, so the trailing `.metrics`\nwas tautological in scrape output and view selectors. Instrument names\nare unchanged — only the scope/meter names.\n\n## Renames in this PR\n\nCore-nodes processors:\n\n- `attributes.processor.metrics` → `attributes.processor`\n- `debug.processor.pdata.metrics` → `debug.processor.pdata`\n- `temporal_reaggregation.processor.pdata.metrics` →\n`temporal_reaggregation.processor.pdata`\n- `content_router.processor.metrics` → `content_router.processor`\n- `signal_type_router.processor.metrics` →\n`signal_type_router.processor`\n- `log_sampling.processor.pdata.metrics` →\n`log_sampling.processor.pdata`\n- `filter.processor.pdata.metrics` → `filter.processor.pdata`\n- `retry.processor.metrics` → `retry.processor`\n- `fanout.processor.metrics` → `fanout.processor`\n\nCore-nodes exporters:\n\n- `perf.exporter.pdata.metrics` → `perf.exporter.pdata`\n- `topic.exporter.metrics` → `topic.exporter`\n\nCore-nodes receivers (added after the original plan):\n\n- `host_metrics.receiver.metrics` → `host_metrics.receiver`\n\nContrib-nodes:\n\n- `azure_monitor_exporter.metrics` → `azure_monitor_exporter`\n- `resource_validator.processor.metrics` →\n`resource_validator.processor`\n\nValidation crate:\n\n- `validation.exporter.metrics` → `validation.exporter`\n- `fanout.processor.metrics` → `fanout.processor`\n\nDoc-only example tweaks (telemetry-macros):\n\n- `my.metrics` → `my` (rustdoc comment in `metric_set` proc-macro)\n- `perf.exporter.pdata.metrics` → `perf.exporter.pdata` (3 places in\n`crates/telemetry-macros/README.md`)\n\n## Intentionally not renamed\n\nLog event names that share the `*.metrics.*` shape (e.g.\n`azure_monitor_exporter.metrics.collect`,\n`pipeline.metrics.reporting.fail`, `tokio.metrics.reporting.fail`,\n`channel.metrics.reporting.fail`, `node.metrics.reporting.fail`). These\nfollow the existing log-event naming convention preserved by PRs #2888 /\n#2912 and are not metric-set names.\n\n## Breaking change for downstream consumers\n\nAnything that selects metrics by **scope/meter name** must be updated.\nInstrument names are unchanged.\n\nExamples:\n\n- View configurations that filter by `ScopeName` (e.g. `ScopeName:\n\"topic.exporter.metrics\"` → `ScopeName: \"topic.exporter\"`).\n- Prometheus relabeling/alerting that keys off the `set=\"…\"` label.\n- Dashboards or queries that group by OTLP scope name.\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2026-05-11T17:22:03Z",
          "tree_id": "b9c052a90a7f2318c5713ce0b0ef30659a51fcd8",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/d83298856be7ea8aca91e00a095a5861d6629389"
        },
        "date": 1778524943918,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1100,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 5.786734688680725,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 6.317451360359662,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.771614583333335,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.27734375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 503.40998270261434,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 6040.919792431372,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.006756,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 213566.0797203464,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 174918.83641798815,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "sapatrjv@gmail.com",
            "name": "sapatrjv",
            "username": "sapatrjv"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "efad8b7b360246b4e5db2384d408981860814f72",
          "message": "Point to Weaver main branch to include gix fix to not build openssl on windows platform and fix tag parsing issue causing crashes in latest weaver release bit. (#2910)\n\n# Change Summary\nPoint to Weaver main branch to include gix fix to not build openssl on\nwindows platform and fix tag parsing issue causing crashes in latest\nweaver release bit.\n\n## What issue does this PR close?\n\nPart of https://github.com/open-telemetry/otel-arrow/issues/2697\n\n## How are these changes tested?\n\nSearch cargo tree and check on windows platform no openssl dependency.\nRan Cargo xtast check\n\n## Are there any user-facing changes?\n\n <!-- If yes, provide further info below -->",
          "timestamp": "2026-05-12T01:20:06Z",
          "tree_id": "44471451669bce2dd3ec03df25f7915640679e51",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/efad8b7b360246b4e5db2384d408981860814f72"
        },
        "date": 1778554012545,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1091.666748046875,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 5.78657754204489,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 6.203130273240963,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.735286458333334,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.1171875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 511.96827503255713,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 6100.9552774713065,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003718,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 214166.75313052666,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 175547.221684819,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
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
          "id": "e2539a0eee3a3f1bf3ed863852f54fcf8b3d9284",
          "message": "Rename otlp.receiver meter to receiver.otlp (#2928)\n\nUsing this small, fully self-contained rename to gauge agreement on the\ndirection proposed in #2926 (just opened). If folks are aligned with the\nURN-order convention, I'll follow up with the remaining per-component\nPRs (core-nodes receivers/processors/exporters, contrib-nodes,\nvalidation, docs sweep). If not, this is small enough to revert.\n\nFlips the OTLP receiver meter/scope name from\n`<component_name>.<component_kind>` to\n`<component_kind>.<component_name>` so it matches the order used by\ncomponent URNs (`urn:otel:receiver:otlp`).\n\n- `otlp.receiver` → `receiver.otlp`\n\nPattern mirrors the previous `.metrics` suffix removal series (#2879 /\n#2888 / #2912 / #2917). Instrument names are unchanged — only the\nscope/meter name.\n\n## Breaking change for downstream consumers\n\nAnything that selects metrics by **scope/meter name** must be updated.\nInstrument names are unchanged.\n\nExamples that need updating:\n\n- View configurations that filter by `ScopeName` (e.g. `ScopeName:\n\"otlp.receiver\"` → `ScopeName: \"receiver.otlp\"`).\n- Prometheus alerting/relabeling that keys off the `otel_scope_name=\"…\"`\nlabel.\n- Dashboards or queries that group by OTLP scope name.\n\n## Before/after — Prometheus `/metrics` scrape\n\nBefore:\n\n```\nacks_received{otel_scope_name=\"otlp.receiver\",pipeline_id=\"main\",node_id=\"otlp_receiver\",core_id=\"0\"} 1234\nrequests_started{otel_scope_name=\"otlp.receiver\",pipeline_id=\"main\",node_id=\"otlp_receiver\",core_id=\"0\"} 567\nrequest_bytes{otel_scope_name=\"otlp.receiver\",pipeline_id=\"main\",node_id=\"otlp_receiver\",core_id=\"0\"} 9876543\n```\n\nAfter:\n\n```\nacks_received{otel_scope_name=\"receiver.otlp\",pipeline_id=\"main\",node_id=\"otlp_receiver\",core_id=\"0\"} 1234\nrequests_started{otel_scope_name=\"receiver.otlp\",pipeline_id=\"main\",node_id=\"otlp_receiver\",core_id=\"0\"} 567\nrequest_bytes{otel_scope_name=\"receiver.otlp\",pipeline_id=\"main\",node_id=\"otlp_receiver\",core_id=\"0\"} 9876543\n```\n\nOnly the `otel_scope_name` label value changes; metric names and other\nlabels are identical.",
          "timestamp": "2026-05-12T04:43:38Z",
          "tree_id": "6ef48ffb84ee262543334df7e70b03e58472bdff",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/e2539a0eee3a3f1bf3ed863852f54fcf8b3d9284"
        },
        "date": 1778564965087,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1112.0689697265625,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 5.80330867318003,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 6.536368278529981,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.670963541666666,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.66796875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 494.8625844791723,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5998.075808428588,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.008578,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 215890.35836840808,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 177296.54200919686,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "33842784+JakeDern@users.noreply.github.com",
            "name": "Jake Dern",
            "username": "JakeDern"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "efe20b16b73d998aae70fab9ed7769406eac27fa",
          "message": "task(comparison_dashboard): Decouple the site directories from the manifest (#2929)\n\n# Change Summary\n\nThis PR generalizes the site directories a little bit so that we can\npublish them into the benchmarks folder relatively easily for #2901.\n\n## What issue does this PR close?\n\n* Closes #2927\n\n## How are these changes tested?\n\nRe-checking the site locally:\n\n<img width=\"1843\" height=\"517\" alt=\"image\"\nsrc=\"https://github.com/user-attachments/assets/51617898-d550-4393-804a-138bacca235e\"\n/>\n\n\n## Are there any user-facing changes?\n\nNo.\n\n---------\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2026-05-12T05:58:12Z",
          "tree_id": "ac408429e6e91f1923925422260dc3ce04059ded",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/efe20b16b73d998aae70fab9ed7769406eac27fa"
        },
        "date": 1778569145524,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1088.3333740234375,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 5.792230448334187,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 6.348819798917246,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.506510416666668,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.8125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 511.92674328303616,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 6083.39613268008,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.008586,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 214928.72861750895,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 176337.87273662846,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "33842784+JakeDern@users.noreply.github.com",
            "name": "Jake Dern",
            "username": "JakeDern"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "c35370ab3b764eb3265eefcece1900f138761f5e",
          "message": "feat(comparison_dashboard): OTC Logs baseline suites (#2934)\n\n# Change Summary\n\nThis PR adds all of the OTC baseline logs suites for OTAP, OTLP, and\nOTLP HTTP.\n\n## What issue does this PR close?\n\n* Closes #2932\n\n## How are these changes tested?\n\nExecuted the 100k for all suites: \n\n```\n  ┌─────────────────────────────────┬────────────┬────────────┬───────┬───────┐\n  │              suite              │ produced/s │ received/s │ drop% │  dur  │\n  ├─────────────────────────────────┼────────────┼────────────┼───────┼───────┤\n  │ otc_logs_otap_gzip_baseline     │ 99,288     │ 101,901    │ -2.6  │ 20.0s │\n  ├─────────────────────────────────┼────────────┼────────────┼───────┼───────┤\n  │ otc_logs_otap_none_baseline     │ 99,360     │ 101,974    │ -2.6  │ 20.0s │\n  ├─────────────────────────────────┼────────────┼────────────┼───────┼───────┤\n  │ otc_logs_otap_zstd_baseline     │ 99,163     │ 99,163     │ 0.00  │ 20.0s │\n  ├─────────────────────────────────┼────────────┼────────────┼───────┼───────┤\n  │ otc_logs_otlp_gzip_baseline     │ 99,017     │ 99,017     │ 0.00  │ 20.0s │\n  ├─────────────────────────────────┼────────────┼────────────┼───────┼───────┤\n  │ otc_logs_otlp_none_baseline     │ 99,269     │ 99,269     │ 0.00  │ 20.0s │\n  ├─────────────────────────────────┼────────────┼────────────┼───────┼───────┤\n  │ otc_logs_otlp_zstd_baseline     │ 99,343     │ 99,343     │ 0.00  │ 20.0s │\n  ├─────────────────────────────────┼────────────┼────────────┼───────┼───────┤\n  │ otc_logs_otlphttp_none_baseline │ 99,480     │ 99,428     │ 0.05  │ 20.0s │\n  └─────────────────────────────────┴────────────┴────────────┴───────┴───────┘\n```\n\n## Are there any user-facing changes?\n\nNone.\n\n---------\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2026-05-12T15:30:01Z",
          "tree_id": "e80876f179a9400bb87392c53908e21d442a6927",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/c35370ab3b764eb3265eefcece1900f138761f5e"
        },
        "date": 1778605588206,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1100,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 5.78365026171154,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 6.3462392108508014,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.863020833333334,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.05078125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 503.4141773484418,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 6040.970128181301,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.006256,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 213810.102919456,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 175855.20895237048,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
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
          "id": "3e94f1fa256d8ea5acf209853548d6d06efb6d2e",
          "message": "Honor the console_fallback setting for engine.observed_state.logging_events send policy (#2914)\n\n# Change Summary\n\n`console_fallback: false`\n\nwould not disable telemetry from logging calls that overflow, due to a\ndefault setting.\n\nThe correct field `engine.observed_state.logging_events`.\n\nFixes https://github.com/open-telemetry/otel-arrow/issues/2916\n\n---------\n\nCo-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>",
          "timestamp": "2026-05-12T17:04:42Z",
          "tree_id": "2c63d0c929429a5106d76b9dfd6fd7ec16561003",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/3e94f1fa256d8ea5acf209853548d6d06efb6d2e"
        },
        "date": 1778617281592,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1091.666748046875,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 5.811292515780087,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 6.211608133086876,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.28125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.3828125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 511.9266494432456,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 6100.459239198676,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.008597,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 214873.47508289633,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 176277.9745717892,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "122434636+SzymonIwaniuk@users.noreply.github.com",
            "name": "Szymon",
            "username": "SzymonIwaniuk"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "ace49720ad3a5bca65fdb9c72b8a20663e903963",
          "message": "feat(query-engine): add hash functions fnv, murmur3, md5, sha1, sha512, xxh3, xxh128 (#2887)\n\n# Change Summary\n  Add seven hash functions to the OTAP query-engine, per #2834:\n- `md5()`: MD5 digest, backed by DataFusion's built-in\n[`crypto::md5`](https://datafusion.apache.org/user-guide/sql/scalar_functions.html#md5)\nUDF.\n- `sha1()`: SHA-1 digest, implemented as a custom `ScalarUDF` using the\n`sha1` crate (DataFusion has no SHA-1 equivalent).\n- `sha512()`: SHA-512 digest, backed by DataFusion's built-in\n[`crypto::sha512`](https://datafusion.apache.org/user-guide/sql/scalar_functions.html#sha512)\nUDF.\n  - `fnv()`: FNV-1a 64-bit hash, implemented as a custom `ScalarUDF`.\n- `murmur3()`: MurmurHash3 32-bit hash, implemented as a custom\n`ScalarUDF`.\n- `xxh3()`: XXH3 64-bit hash, implemented as a custom `ScalarUDF` using\n`xxhash-rust`.\n- `xxh128()`: XXH3 128-bit hash, implemented as a custom `ScalarUDF`\nusing `xxhash-rust`.\n  ```kql\nlogs | extend attributes[\"hash\"] = encode(md5(attributes[\"body\"]),\n\"hex\")\nlogs | extend attributes[\"bucket\"] = murmur3(attributes[\"service.name\"])\n  logs | extend attributes[\"sig\"] = xxh3(attributes[\"message\"])\n  ```\n  ## Files\n- `consts.rs`, `parser.rs`: register all 7 functions as external\nfunctions with `param_placeholders(1)`.\n- `pipeline/expr.rs`: import DataFusion's `md5()`, `sha512()` and the\nnew custom UDFs, add them to `DataFusionFunctionDef::from_func_name`\nwith `requires_dict_downcast: true`.\n- `pipeline/functions/fnv.rs` (new): custom `FnvHashFunc` - FNV-1a\n64-bit, returns `Int64`.\n- `pipeline/functions/murmur3.rs` (new): custom `Murmur3HashFunc` -\nMurmurHash3 32-bit, returns `Int64`.\n- `pipeline/functions/sha1.rs` (new): custom `Sha1Func` using the `sha1`\ncrate, returns `Binary`.\n- `pipeline/functions/xxh3.rs` (new): custom `Xxh3Func` using\n`xxhash-rust`, returns `Int64`.\n- `pipeline/functions/xxh128.rs` (new): custom `Xxh128Func` using\n`xxhash-rust`, returns `Binary` (16 bytes big-endian).\n- `pipeline/functions.rs`: module declarations and `make_udf_function!`\nregistrations.\n- `crates/query-engine/Cargo.toml`: add `sha1` and `xxhash-rust`\nworkspace dependencies.\n  ## What issue does this PR close?\n  - Closes sub-issue of #2834\n  - Part of #2818\n  ## How are these changes tested?\n  Two layers of tests, all in this PR:\n- Unit tests in each custom UDF module (`fnv`, `murmur3`, `sha1`,\n`xxh3`, `xxh128`): verify scalar string input, binary input, and null\nhandling.\n- `pipeline::assign` end-to-end tests for every function, exercising\nboth OPL and KQL parsers through the full pipeline. Binary-returning\nfunctions (`md5`, `sha1`, `sha512`, `xxh128`) are wrapped with\n`encode(..., \"hex\")` and\nthe output hex string is asserted. Integer-returning functions (`fnv`,\n`murmur3`, `xxh3`) assert the `Int64` value directly.\n  `cargo xtask quick-check` passes clean.\n  ## Are there any user-facing changes?\nYes users of the transform processor / query-engine can now call these\nhash functions in OPL/KQL programs:\n  ```kql\nlogs | extend attributes[\"id\"] = encode(sha1(attributes[\"body\"]), \"hex\")\n  logs | extend attributes[\"bucket\"] = fnv(attributes[\"service.name\"])\n  logs | extend attributes[\"sig\"] = xxh3(attributes[\"message\"])\nlogs | extend attributes[\"hash\"] = encode(xxh128(attributes[\"message\"]),\n\"hex\")\n\n---------\n\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-05-12T17:11:36Z",
          "tree_id": "b4157b764d2fdfda75d53700bc3e13556bf89d90",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/ace49720ad3a5bca65fdb9c72b8a20663e903963"
        },
        "date": 1778623093708,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1108.4744873046875,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 5.731044556863988,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 6.0817083172147,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.135807291666666,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.44921875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 503.43742534287804,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 6083.913292702916,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003485,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 214672.30594431717,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 176031.8091838065,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "33842784+JakeDern@users.noreply.github.com",
            "name": "Jake Dern",
            "username": "JakeDern"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "a9e6735caf596fcd1c903efa5004b933d0b18593",
          "message": "feat(comparison_dashboard): Add a workflow for building/publishing the comparison dashboard when code is updated on main (#2937)\n\n# Change Summary\n\nThis is the first part of #2901 which is to automatically build and\npublish the site.\n\nWe also need to add a workflow to the benchmarks branch that triggers\nwhen there are changes to the data there.\n\n## What issue does this PR close?\n\n* Part of #2901\n\n## How are these changes tested?\n\nBeen testing these on my fork - Seems to work there, fingers crossed.\n\nExample run:\nhttps://github.com/JakeDern/otel-arrow/actions/runs/25747403028\n\n## Are there any user-facing changes?\n\nYes, this will start publishing the stub site at\n`https://open-telemetry.github.io/otel-arrow`",
          "timestamp": "2026-05-12T17:25:38Z",
          "tree_id": "30646c98969fc7b05f22ffb866fc122e8c65f83a",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/a9e6735caf596fcd1c903efa5004b933d0b18593"
        },
        "date": 1778626551694,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1100,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 5.765145113841364,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 6.198786377708979,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.431640625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.1015625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 503.31593193365137,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 6039.791183203817,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.017969,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 215011.83676922816,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 176378.07905706388,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "30638925+marcsnid@users.noreply.github.com",
            "name": "Marc Snider",
            "username": "marcsnid"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "cbc931a0d80280406ba34276e6d898848df348c5",
          "message": "Reorganize query-engine languages and inline OTLP bridge (#2797)\n\n# Change Summary\n\nRenames the `opl` crate to `query-engine-languages` and nests OPL as a\nsubmodule inside it. Adds the OTTL parser stub (from\n`experimental/query_engine/ottl-parser`) as a second submodule, giving\nus a single home for language parsers going forward. Also pulls\n`engine-recordset-otlp-bridge` directly into `recordset_kql_processor`\nas a local `otlp_bridge` module so contrib-nodes doesn't depend on that\nexternal crate anymore.\n\nAll existing tests pass, no behavior changes.\n\n## What issue does this PR close?\n\n* Closes #2743\n\n## How are these changes tested?\n\nExisting unit tests cover all moved code. `cargo xtask check` passes\n(structure, fmt, clippy, full test suite).\n\n## Are there any user-facing changes?\n\nNo.",
          "timestamp": "2026-05-12T19:08:56Z",
          "tree_id": "dc7ac1e4846443d233ff7dd3583857225ddc4555",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/cbc931a0d80280406ba34276e6d898848df348c5"
        },
        "date": 1778629964240,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1110.3448486328125,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 5.776285937032691,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 6.2712572533849125,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.085416666666667,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.7890625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 494.9044968979808,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5990.05097969625,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003496,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 214544.29856829223,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 175866.66041322457,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "Tom.Tan@microsoft.com",
            "name": "Tom Tan",
            "username": "ThomsonTan"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "20b6b1829c6ad2ff768eca67458460952d4fb368",
          "message": "docs: rename enginectl.md to dfctl.md (#2863)\n\n# Change Summary\n\nThe doc seems named after the crate (`otap-df-enginectl`) but its title,\nevery command example, and the link label in `docs/admin/README.md` all\nalready used `dfctl` -- the actual binary name. Align the filename with\nthe user-facing tool.\n\n## What issue does this PR close?\n\n<!--\nWe highly recommend correlation of every PR to an issue\n-->\n\n* Closes #NNN\n\n## How are these changes tested?\n\n## Are there any user-facing changes?\n\nNo code changes, only doc file name changed.\n\n---------\n\nCo-authored-by: Joshua MacDonald <jmacd@users.noreply.github.com>",
          "timestamp": "2026-05-12T19:18:40Z",
          "tree_id": "a45239118774d97bd14e8b324a4bc6ce26ecc1e7",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/20b6b1829c6ad2ff768eca67458460952d4fb368"
        },
        "date": 1778631016678,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1108.4744873046875,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 5.776853550526758,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 6.237362841445485,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.992708333333333,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.46484375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 503.1780102148068,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 6080.778326833173,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.03442,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 215234.87309971626,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 176608.43828595147,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
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
          "id": "e090cb8a8cca74f335d16690152e78b02531934c",
          "message": "Rename remaining meters to URN order: <kind>.<name> (#2935)\n\nFollow-up to #2928 to flip every remaining meter/scope name from\n`<component_name>.<component_kind>` to\n`<component_kind>.<component_name>`, matching the order used by\ncomponent URNs (`urn:otel:<kind>:<name>`).\n\nCloses #2926.\n\n## Renames in this PR\n\nReceivers:\n\n- `traffic_generator.receiver` → `receiver.traffic_generator`\n- `topic.receiver` → `receiver.topic`\n- `otap.receiver` → `receiver.otap`\n- `syslog_cef.receiver` → `receiver.syslog_cef`\n- `host_metrics.receiver` → `receiver.host_metrics`\n\nProcessors:\n\n- `attributes.processor` → `processor.attributes`\n- `debug.processor.pdata` → `processor.debug.pdata`\n- `temporal_reaggregation.processor.pdata` →\n`processor.temporal_reaggregation.pdata`\n- `content_router.processor` → `processor.content_router`\n- `signal_type_router.processor` → `processor.signal_type_router`\n- `log_sampling.processor.pdata` → `processor.log_sampling.pdata`\n- `filter.processor.pdata` → `processor.filter.pdata`\n- `retry.processor` → `processor.retry`\n- `fanout.processor` → `processor.fanout`\n- `transform.processor` → `processor.transform`\n- `resource_validator.processor` → `processor.resource_validator`\n\nExporters:\n\n- `perf.exporter.pdata` → `exporter.perf.pdata`\n- `topic.exporter` → `exporter.topic`\n- `validation.exporter` → `exporter.validation`\n- `azure_monitor_exporter` → `exporter.azure_monitor` (also splits the\npreviously underscore-joined token)\n\nThe `.pdata` qualifier is preserved as a trailing suffix where present.\n\n## Out of scope\n\nAlready kind-first or kind-in-middle (left as-is): `engine`, `pipeline`,\n`pipeline.runtime_control`, `pipeline.completion`, `tokio.runtime`,\n`flow`, `processor.compute`, `exporter.pdata`, `channel.sender`,\n`channel.receiver`, `node.consumer`, `node.producer`,\n`node.completion_emission`, `node.local_wakeup.scheduler`. The\n`otap.<kind>.<name>` signal-namespaced sets (`otap.exporter.geneva`,\n`otap.exporter.grpc.async`, `otap.exporter.parquet`,\n`otap.processor.batch`, `otap.processor.durable_buffer`) are also left\nas-is — restructuring `<signal>.<kind>.<name>` is a separate decision.\n\nLog event names that share the `*.metrics.*` shape (e.g.\n`azure_monitor_exporter.metrics.collect`,\n`pipeline.metrics.reporting.fail`) are intentionally preserved — they\nare log events, not meter names, matching the convention kept by #2888 /\n#2912 / #2917.\n\n## Breaking change for downstream consumers\n\nAnything that selects metrics by **scope/meter name** must be updated.\nInstrument names are unchanged.\n\nExamples:\n\n- View configurations that filter by `ScopeName` (e.g. `ScopeName:\n\"topic.exporter\"` → `ScopeName: \"exporter.topic\"`).\n- Prometheus alerting/relabeling that keys off the `otel_scope_name=\"…\"`\nlabel.\n- Dashboards or queries that group by OTLP scope name.",
          "timestamp": "2026-05-12T20:25:55Z",
          "tree_id": "90b834d38af2af8bf997ea58dc95cf12a706bc9f",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/e090cb8a8cca74f335d16690152e78b02531934c"
        },
        "date": 1778632059547,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1111.8643798828125,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 5.795373728881606,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 6.416248935676136,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.360677083333332,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.44140625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 503.39343131230123,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 6100.4458201406,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.008729,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 216569.6933043563,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 177832.05307982297,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "Tom.Tan@microsoft.com",
            "name": "Tom Tan",
            "username": "ThomsonTan"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "1f4ee75bbc10cd4c6453839b7d6b0834484bce14",
          "message": "feat(query-engine): fast-path is_type checks on AnyValue columns (#2861)\n\n# Change Summary\n\nPreviously `is_type` on a heterogeneous `AnyValue` struct column went\nthrough the generic path: split the column by concrete subtype, evaluate\nthe UDF on each homogeneous segment, then stitch the boolean results\nback together. For a check that only depends on the discriminator\ncolumn, this is wasteful.\n\n## What issue does this PR close?\n\n<!--\nWe highly recommend correlation of every PR to an issue\n-->\n\n* Closes #2841 \n\n## How are these changes tested?\nUnit-test\n\n## Are there any user-facing changes?\n\nYes, user-facing behavior changes in the OTAP query engine:\n\nThese now work for AnyValue attributes via the discriminator-column fast\npath\n\n```\nlogs | where attributes[\"complex\"] is Array\nlogs | where attributes[\"complex\"] is Map\nlogs | where attributes[\"complex\"] is Null\n```\n\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-05-12T21:57:31Z",
          "tree_id": "6e55c0682b106b61952755edca6a70ce7bb68086",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/1f4ee75bbc10cd4c6453839b7d6b0834484bce14"
        },
        "date": 1778633105215,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1088.1356201171875,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 5.757148140063513,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 6.351000076869859,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.698307291666666,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.99609375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 503.41197095602234,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5981.216807460536,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.006519,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 213487.84351467012,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 174884.38170983005,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "33842784+JakeDern@users.noreply.github.com",
            "name": "Jake Dern",
            "username": "JakeDern"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "353bd54f7944285d46e81b806a0eef7001d055cf",
          "message": "feat(comparison_dashboard): Add all the dfe suites and define a DFE engine baseline comparison (#2943)\n\n# Change Summary\n\nThis PR adds all the DFE baseline logs suites to the dashboard and\ndefines a comparison of all the DFE baseline suites for further testing\nof the site. No data checked in yet so it will look blank until we add\nsome.\n\n## What issue does this PR close?\n\n* Closes #2892\n\n## How are these changes tested?\n\nLocally: \n\n<img width=\"2449\" height=\"994\" alt=\"image\"\nsrc=\"https://github.com/user-attachments/assets/d362d7de-194f-4a1d-a5ec-e29dc8bfdda0\"\n/>\n\n## Are there any user-facing changes?\n\nYes, stub of a DFE comparison.",
          "timestamp": "2026-05-12T22:58:48Z",
          "tree_id": "03dcf0ec421c6830baa66c6734ad1bd698318653",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/353bd54f7944285d46e81b806a0eef7001d055cf"
        },
        "date": 1778634152356,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1100,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 5.78596636498044,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 6.186650173946656,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.9328125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.94140625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 503.43437135174446,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 6041.212456220933,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003849,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 215888.6532276334,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 177201.8611746481,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "Tom.Tan@microsoft.com",
            "name": "Tom Tan",
            "username": "ThomsonTan"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "500749c790fb8b702146dddaa51f06e3a1a5ecb4",
          "message": "fix(load_generator): honor --serve-port in pre-flight port check (#2940)\n\n# Change Summary\n\n`loadgen.py --serve` always probed the hard-coded `FLASK_PORT` (5001)\nbefore calling `app.run(port=args.serve_port)`, so `--serve-port` was\neffectively ignored by the pre-flight check. Three observable failure\nmodes:\n\n- `--serve-port 8080` with 8080 busy: check passed, then Flask died with\na late `OSError: address already in use`.\n- `--serve-port 8080` with an unrelated process on 5001: check raised,\nrefusing to start a server that would have bound fine.\n- Error message hard-coded `FLASK_PORT` in the string, so it named the\nwrong port either way.\n\nAlso tightened `is_port_in_use` itself: it used `connect_ex` against\n`0.0.0.0`, which only sees loopback listeners and tests reachability\ninstead of bindability -- the property actually relevant to `app.run`.\n\n## What issue does this PR close?\n\n<!--\nWe highly recommend correlation of every PR to an issue\n-->\n\n* Closes #NNN\n\n## How are these changes tested?\n\nTested locally.\n\n## Are there any user-facing changes?\n\n <!-- If yes, provide further info below -->\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2026-05-12T23:52:03Z",
          "tree_id": "40ccb38635bfc8549ec37f920d3de4b2498fec9a",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/500749c790fb8b702146dddaa51f06e3a1a5ecb4"
        },
        "date": 1778635197575,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1080,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 5.820038599199364,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 6.267893312717433,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.345833333333335,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.06640625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 511.9403674795281,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 6040.896336258432,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.006989,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 215500.67737105218,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 177127.6639120992,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "33842784+JakeDern@users.noreply.github.com",
            "name": "Jake Dern",
            "username": "JakeDern"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "db316a8aff1b939ad6fe495fca7d948f7e9d1218",
          "message": "feat(comparison_dashboard): Run validate command on PRs (#2944)\n\n# Change Summary\n\nAdds a workflow for running some comparison dashboard validations.\nTriggered on PR, but I think still needs to be added as required in the\nrepo settings to actually block stuff unless we'd rather leave it as an\noptional check.\n\n## What issue does this PR close?\n\n* Closes #2869 \n\n## How are these changes tested?\n\nLoosely\n\n## Are there any user-facing changes?\n\nYes, new CI check.\n\n---------\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2026-05-12T23:56:23Z",
          "tree_id": "f5abe104ab8ca7f109e8f2b265d2ce30212c97b3",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/db316a8aff1b939ad6fe495fca7d948f7e9d1218"
        },
        "date": 1778636463909,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1129.310302734375,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 5.7882575703090335,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 6.286868937689144,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.026692708333332,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.8359375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 494.87951518605684,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 6083.605074614802,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.006525,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 214619.1557834158,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 176022.9862742083,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "cithomas@microsoft.com",
            "name": "Cijo Thomas",
            "username": "cijothomas"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "0c37386ba3e4cc9957ddb5a2b6a95fd6e2fa1752",
          "message": "fix(engine): centralize telemetry timer management in runtime manager (#2804)\n\nFixes https://github.com/open-telemetry/otel-arrow/issues/1305\n\n## Motivation\n\nOur idle perf tests were showing surprisingly high CPU usage.\nInvestigation revealed that each node was independently calling\n`start_periodic_telemetry(Duration::from_secs(1))`, triggering internal\nmetric collection (syscalls like `getrusage`, jemalloc stats, tokio\nworker metrics, channel snapshots) every second across all nodes. At 1s\nintervals, the telemetry overhead itself dominated idle CPU\nmeasurements, producing misleading results.\n\n## Changes\n\nCentralizes telemetry timer registration in the runtime control manager\nso all nodes use the configured `engine.telemetry.reporting_interval`\ninstead of each node managing its own timer independently. This removes\n~250 lines of per-node boilerplate across 15 nodes, with shutdown\ncancellation handled centrally.\n\nAlso bumps the idle perf test to `reporting_interval: 5s` (with matching\n5s Prometheus scrape) for a more realistic deployment baseline. With\ncentralized timing and the 5s interval, idle CPU numbers look\nsignificantly better — engine CPU drops ~2.3x and pipeline CPU drops\n~6.4x compared to the previous 1s configuration.\n\nAdds validation rejecting zero-duration `reporting_interval` to prevent\naccidental spin loops.\n\n**Notes**\n- `perf_exporter.config.frequency` is now silently ignored;\ndeprecation/cleanup is a follow-up.\n- `start_periodic_telemetry` API is still public — TBD whether to\nremove.\n- TODO comment added in `pipeline_ctrl.rs` for an\neager-timer-registration race flagged in review (telemetry can queue\nahead of `Shutdown` in a slow-starting node's bounded control channel);\nto be addressed via a node-ready signal in a follow-up.\n\n---------\n\nCo-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2026-05-13T00:09:18Z",
          "tree_id": "9f4b8b786a51649aafd8528ac6f7e3055f956f5d",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/0c37386ba3e4cc9957ddb5a2b6a95fd6e2fa1752"
        },
        "date": 1778643207472,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1080,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 5.775409019288323,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 6.208669507942658,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.571354166666666,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.6875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 511.9739149290344,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 6041.292196162605,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003057,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 215054.38206049547,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 176416.5105194937,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "33842784+JakeDern@users.noreply.github.com",
            "name": "Jake Dern",
            "username": "JakeDern"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "f1a007ec383bd70570a1d55f97fb970bd4baf08f",
          "message": "feat(comparison_dashboard): Add DFE metrics and traces baseline suites (#2950)\n\n# Change Summary\n\nThis PR adds metrics and traces baseline DFE suites\n\n## What issue does this PR close?\n\n* Closes #2947 \n\n## How are these changes tested?\n\n100k smoke test for every added suite.\n\n## Are there any user-facing changes?\n\nNo.",
          "timestamp": "2026-05-13T00:40:58Z",
          "tree_id": "7dae2eeb401a78dfc88d5125fbbe641902af2f29",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/f1a007ec383bd70570a1d55f97fb970bd4baf08f"
        },
        "date": 1778644245580,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1079.6610107421875,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 5.763688497232001,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 6.238613861386138,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.322005208333334,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.73828125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 503.32043530286336,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5937.474965606659,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.017432,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 213615.74014503317,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 174871.6880136071,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "cithomas@microsoft.com",
            "name": "Cijo Thomas",
            "username": "cijothomas"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "b645a269cc18cba6310ca15648a990f4bdc94e68",
          "message": "fix: restore uncapped throughput in traffic generator (#2946)\n\nPR #2723 broke uncapped mode — saturation tests dropped from ~290K to\n~1.5K logs/sec. This restores the original behavior.",
          "timestamp": "2026-05-13T00:41:34Z",
          "tree_id": "0dcd4ec18b4478c57fde24dcce2c41b41daee571",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/b645a269cc18cba6310ca15648a990f4bdc94e68"
        },
        "date": 1778645988531,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1136.151611328125,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 95.91028054800113,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.16134432861365,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 32.581380208333336,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 33.79296875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 43900.95107894757,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 542682.310713521,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004167,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 15386134.543090884,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 15342412.473993365,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "lalit_fin@yahoo.com",
            "name": "Lalit Kumar Bhasin",
            "username": "lalitb"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "4fced8c19ccd3ed3cb67d357c4ffa3d0b859733f",
          "message": "Add Linux user_events receiver (#2787)\n\n# Change Summary\n\nSplit out from the original combined PR #2717.\n\n  Adds a contrib receiver for Linux user_events tracepoints.\n\n  The receiver supports two decode formats:\n- `tracefs`: generic Linux tracepoint decoding based on the static\nschema exposed in tracefs format files.\n- `event_header`: decodes self-describing EventHeader payloads.\nEventHeader comes from the Microsoft LinuxTracepoints-Rust project, but\nis open and usable by any producer.\n\nThe receiver uses `one_collect` for `perf`/`user_events` collection, and\nsupports single or multiple subscriptions, batching, late registration,\nand basic receiver metrics. It is behind the `userevents-receiver`\nfeature flag.\n\nAlso adds a Linux CI smoke test that registers and reads both tracefs\nand EventHeader user_events tracepoints when kernel support is\navailable.\n\n\n## What issue does this PR close?\n\n* Closes #2698\n\n## How are these changes tested?\n\nAdded Linux CI smoke test\n\n## Are there any user-facing changes?\n\nYes, a receiver.\n\n---------\n\nCo-authored-by: Utkarsh Umesan Pillai <66651184+utpilla@users.noreply.github.com>",
          "timestamp": "2026-05-13T17:49:35Z",
          "tree_id": "7f8242c1c6e4f2f7eac4254410c53d7a7c6272f1",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/4fced8c19ccd3ed3cb67d357c4ffa3d0b859733f"
        },
        "date": 1778697787064,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1100.4676513671875,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.80296270799728,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.38197217928904,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 32.24296875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 32.90625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 45605.83548849392,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 547483.336598543,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.006356,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 15552943.748627268,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 15522105.706624521,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
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
          "id": "cf292e5254e092ed378f23502d44a3be975ff67b",
          "message": "chore(deps): update geneva-uploader digest to 24b5ae5 (#2915)\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n|---|---|---|---|\n| geneva-uploader | workspace.dependencies | digest | `ce866b4` →\n`24b5ae5` |\n\n---\n\n### Configuration\n\n📅 **Schedule**: (UTC)\n\n- Branch creation\n  - At any time (no schedule defined)\n- Automerge\n  - At any time (no schedule defined)\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My4xNTkuMiIsInVwZGF0ZWRJblZlciI6IjQzLjE1OS4yIiwidGFyZ2V0QnJhbmNoIjoibWFpbiIsImxhYmVscyI6WyJkZXBlbmRlbmNpZXMiXX0=-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: Lalit Kumar Bhasin <lalit_fin@yahoo.com>\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-05-13T22:48:27Z",
          "tree_id": "f78add47efb80ec472665bff14510068d7ba5c6d",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/cf292e5254e092ed378f23502d44a3be975ff67b"
        },
        "date": 1778716292941,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1121.0526123046875,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.28233957505603,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.40350893478764,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 32.30260416666667,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 33.125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 44574.39682103506,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 544276.8453936913,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.016516,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 15461939.593305077,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 15412893.47028589,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "33842784+JakeDern@users.noreply.github.com",
            "name": "Jake Dern",
            "username": "JakeDern"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "158ad9d9ccf91af559e3972a250f94e0e17e8c26",
          "message": "feat(comparison_dashboard): Record, display, and validate environment details for suite runs (#2958)\n\n# Change Summary\n\nThis PR adds environment details like cpu architecture/model, available\ncore counts/memory, os, and run start/end times.\n\nWe also validate at dashboard build time that the environment for all\ntests within a comparison was the same and fail the build if not.\n\n## What issue does this PR close?\n\n* Closes #2949\n\n## How are these changes tested?\n\n<img width=\"2397\" height=\"1960\" alt=\"image\"\nsrc=\"https://github.com/user-attachments/assets/df37ed3b-364d-4ee1-ab80-87a705bd43dd\"\n/>\n\n## Are there any user-facing changes?\n\nYes, new information in the dashboard.\n\n---------\n\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-05-13T23:03:36Z",
          "tree_id": "5fb9153454db354c5f277288d1d3f471e9cc1355",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/158ad9d9ccf91af559e3972a250f94e0e17e8c26"
        },
        "date": 1778717331598,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1116.6824951171875,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.15369945748226,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.2371627690878,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 32.721614583333334,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 33.58203125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 44752.692479123245,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 544498.2022030982,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.006222,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 15510154.618027855,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 15460977.95559286,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "33842784+JakeDern@users.noreply.github.com",
            "name": "Jake Dern",
            "username": "JakeDern"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "55099cb930c81eb11c892e263d75ffcdb071ed08",
          "message": "feat(comparison_dashboard): Add OTC metrics and traces baselines (#2959)\n\n# Change Summary\n\nAdds the OTC metrics and traces baselines to mirror the logs ones.\n\n## What issue does this PR close?\n\n* Closes #2948\n\n## How are these changes tested?\n\nSmoke tested the runs locally.\n\n## Are there any user-facing changes?\n\nNew suites, but not really as they're not visible yet.",
          "timestamp": "2026-05-13T23:04:26Z",
          "tree_id": "cb9fa2580392e6ead6a893b47dda12fa66ea612b",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/55099cb930c81eb11c892e263d75ffcdb071ed08"
        },
        "date": 1778718373193,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1099.518798828125,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 95.9174769946192,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 99.78688961139495,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 31.985416666666666,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 32.88671875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 44326.038289501936,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 531699.1484139198,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.006265,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 15224831.65927939,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 15181203.147154313,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
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
          "id": "f0b1ca6c8504355628090d13bcb6490ba7a80664",
          "message": "Remove stale TODOs from OTLP Http Exporter that are already implemented (#2969)\n\n# Change Summary\n\nThe module-level `ToDo:` block at the top of\n`crates/core-nodes/src/exporters/otlp_http_exporter/mod.rs` still listed\ntwo items that have since been implemented. This PR removes them so the\nlist reflects only work that is actually outstanding.\n\n## Removed entries and where they are implemented\n\n### 1. `TLS/mTLS`\n\n- Configured via `HttpClientSettings.tls: Option<TlsClientConfig>` in\n`crates/otap/src/otlp_http/client_settings.rs` (lines 51-53).\n- Trust store (CA file/PEM, system roots), `insecure_skip_verify`, and\nthe mTLS client identity (cert + key paired into a reqwest `Identity`)\nare wired into the `reqwest::ClientBuilder` at\n`crates/otap/src/otlp_http/client_settings.rs` (lines 85-188).\n- Exporter validates and surfaces TLS-specific configuration errors at\n`crates/core-nodes/src/exporters/otlp_http_exporter/mod.rs` (lines\n139-175), e.g. unsupported `server_name_override`, ignored `insecure:\ntrue` when scheme is `https://`.\n- Integration coverage in `crates/otap/tests/otlp_exporter_tls.rs`:\n  - `otlp_exporter_connects_with_mtls`\n  - `otlp_exporter_connects_with_tls_only`\n  - `otlp_exporter_fails_partial_mtls`\n  - `otlp_exporter_fails_with_invalid_ca_pem`\n  - `otlp_exporter_allows_http_with_tls_config`\n- Proxy + TLS combinations in\n`crates/otap/tests/otlp_exporter_proxy_tls.rs`.\n\n### 2. `Allow endpoint overrides for each signal type (similar to Go\ncollector implementation)`\n\n- Per-signal override fields exist on the config struct:\n`traces_endpoint`, `metrics_endpoint`, `logs_endpoint` in\n`crates/core-nodes/src/exporters/otlp_http_exporter/config.rs` (lines\n26-40).\n- Resolved per-signal at construction time at\n`crates/core-nodes/src/exporters/otlp_http_exporter/mod.rs` (lines\n156-167 and 203-221), then dispatched per `SignalType` in the export\npath at lines 368-370 of the same file.\n\n## Remaining ToDo items (unchanged)\n\n- Proxy settings\n- Compression (payloads and accepting compressed responses)\n- JSON encoding payloads (only proto is supported, not configurable; see\nexisting inline TODO at\n`crates/core-nodes/src/exporters/otlp_http_exporter/mod.rs` line 670)\n- Unit test metrics reporting\n\n## What issue does this PR close?\n\n* Closes #NNN\n\n## How are these changes tested?\n\n## Are there any user-facing changes?\n\n <!-- If yes, provide further info below -->\n\nCo-authored-by: Utkarsh Umesan Pillai <utpilla@users.noreply.github.com>\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-05-14T00:30:31Z",
          "tree_id": "32472846cecce6c9b48427b60efe12b74a8eedff",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/f0b1ca6c8504355628090d13bcb6490ba7a80664"
        },
        "date": 1778721388859,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1120.076416015625,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.15608385046556,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.14895527008203,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 31.828645833333333,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 32.765625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 44626.39171034643,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 544476.1099497525,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003955,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 15471626.692371638,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 15422949.64242879,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
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
          "id": "dbd487cad167f94df0ea1f00758212aeb9293163",
          "message": "feat: add num_connections config to OTLP gRPC exporter (#2967)\n\n## Summary\n\nAdd a `num_connections` configuration option to the OTLP gRPC exporter\nthat controls how many independent TCP connections (tonic Channels) are\ncreated per pipeline.\n\nFixes https://github.com/open-telemetry/otel-arrow/issues/1323\n\n## Problem\n\nWhen the receiver uses `SO_REUSEPORT` across multiple cores, the kernel\ndistributes **new TCP connections** (not individual RPCs) across\nlistener sockets. With the previous behavior of 1 gRPC channel per\npipeline, this caused severe core imbalance — e.g., with 2 engine cores:\none core at 60% and another at 94%.\n\n## Solution\n\n- Added `num_connections` config field (default: 1) to the OTLP gRPC\nexporter\n- When `num_connections > 1`, creates N independent tonic Channels, each\nestablishing its own TCP connection\n- Rewrote `GrpcClientPool` to use a FIFO `VecDeque` for round-robin\ndistribution of gRPC clients across channels\n- Pool is sized to `max(max_in_flight, num_connections)` ensuring every\nchannel gets at least one client\n- Updated saturation test templates to set `num_connections = num_cores\n* 4`\n\n## Results\n\nWith `num_connections` set appropriately:\n- Core imbalance fixed: 60%/94% → 99%/99%\n- 2-core throughput improved from 0.90× to 1.36× of 1-core baseline\n\n| Config | logs/sec | Scaling | Core balance |\n|--------|----------|---------|--------------|\n| 1-core, 1 conn (old) | 164,727 | baseline | N/A |\n| 2-core, 1 conn (old) | 148,685 | 0.90× | 60%/94% |\n| 1-core, 4 conns (new) | 177,461 | baseline | 99.6% |\n| 2-core, 8 conns (new) | 241,964 | 1.36× | 95.4% avg, balanced |\n\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-05-14T00:38:08Z",
          "tree_id": "a4279e76f65c3be4922d35583f6f3ef2275e5823",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/dbd487cad167f94df0ea1f00758212aeb9293163"
        },
        "date": 1778727812827,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1120.801513671875,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.25005357396212,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.17883809081766,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 32.733984375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 33.58984375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 44712.24475340919,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 545847.7665793103,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00325,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 15524991.625542825,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 15475085.804392053,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
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
          "distinct": false,
          "id": "bdb39841a9ab8bb46062349d1b156ccb86d04482",
          "message": "fix(admin): group Prometheus metrics contiguously per spec (#2956)\n\n## Problem\n\nThe Prometheus exposition format requires all sample lines for a given\nmetric name to appear as one contiguous group, preceded by at most one\n`# HELP` and one `# TYPE` directive. The previous implementation\niterated per-entity, emitting each entity's full metric set inline. With\nN entities sharing the same metric set (e.g. 10 pipeline-thread cores),\nevery metric was re-opened N times non-contiguously. Strict parsers\n(like `prometheus_client.parser.text_string_to_metric_families`) split\nre-opened groups into separate unknown-type families, causing multi-core\nmetrics like `logs_produced` to report only one core's throughput.\n\n## Solution\n\nReplace the direct-emission approach with a **two-phase\ncollect-then-emit** pattern:\n\n1. **Collect phase**: During the registry visit, samples are grouped by\nmetric name into a `PromGroupedMetrics` structure that preserves\ninsertion order.\n2. **Emit phase**: After all entities are visited, each metric family is\nemitted as a contiguous block: `# HELP` → `# UNIT` → `# TYPE` → all\nsample lines.\n\nBoth `format_prometheus_text` and `agg_prometheus_text` are updated.\n\n## Testing\n\n- All 50 existing + new tests pass (`cargo xtask check` clean)\n- Added `test_format_prometheus_text_multi_entity_contiguous_grouping`\nthat registers two entities sharing a metric set and verifies:\n  - Exactly one `# HELP` and `# TYPE` per metric name\n- All lines (directives + samples) for each metric are contiguous (no\ngaps)\n  - Both entities' samples are present\n\nFixes #2945\n\nCo-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>",
          "timestamp": "2026-05-14T06:18:43Z",
          "tree_id": "5d32ce4a6f2c44c7628912de1830bfa9925102c9",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/bdb39841a9ab8bb46062349d1b156ccb86d04482"
        },
        "date": 1778742408680,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -2.0985770225524902,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.07148115975235,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.31899713688772,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 32.52200520833333,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 33.2734375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 530620.1288772116,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 541755.5999423361,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00285,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 15383539.635008264,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 15337969.33462937,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
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
          "id": "77d6150a78e6580d2cbdc011304eabba39ffee68",
          "message": "feat(perf): add syslog TCP perf test through OTel Collector (#2962)\n\nAdd syslog TCP perf test suite routing traffic through the Go OTel\nCollector with syslog receiver, for comparison against the existing\ndf-engine syslog path.",
          "timestamp": "2026-05-14T06:25:15Z",
          "tree_id": "747800e13b64c3d60ae9e414886a100a91f16d7c",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/77d6150a78e6580d2cbdc011304eabba39ffee68"
        },
        "date": 1778743450894,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.3640714883804321,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.00441910272482,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.47395665634676,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 32.96302083333333,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 34.05078125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 534850.0876443463,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 542145.8251193928,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002159,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 15381890.889938988,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 15338070.399874859,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "33842784+JakeDern@users.noreply.github.com",
            "name": "Jake Dern",
            "username": "JakeDern"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "a47b96ab5d4676fdcc4717b3f68e817dea1804b3",
          "message": "fix(comparison_dashboard): Properly collect engine metrics for the go collector (#2971)\n\n# Change Summary\n\nUpdate the sql reports to grab metrics for OTC.\n\n## What issue does this PR close?\n\n* Closes #2855\n\n## How are these changes tested?\n\n<img width=\"2649\" height=\"1523\" alt=\"image\"\nsrc=\"https://github.com/user-attachments/assets/625e45a5-107c-4290-8ac0-f96470b6dbce\"\n/>\n\n## Are there any user-facing changes?\n\nNo.",
          "timestamp": "2026-05-14T14:59:00Z",
          "tree_id": "7d8b2cf2091bba2ededfa610e4da2794e09ea3a6",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/a47b96ab5d4676fdcc4717b3f68e817dea1804b3"
        },
        "date": 1778774578430,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.07829627394676208,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.17305881800974,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.11312480644163,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 31.838541666666668,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 32.5859375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 544862.4676505657,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 545289.0746562426,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.008391,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 15511411.003040247,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 15463061.97168377,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
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
          "distinct": false,
          "id": "db9062212d3b6f4de5f72e1b1c875ed24b85f81b",
          "message": "Trim no-batch baselines from nightly batch processor perf suite (#2972)\n\n## Motivation\n\nThe Nightly Batch Processor perf suite\n(`tools/pipeline_perf_test/test_suites/integration/nightly/batch-processor-docker.yaml`)\ncurrently runs 12 tests = 3 signals (logs/metrics/traces) × 4 pipeline\nconfigs (OTAP+batch, OTAP-no-batch, OTLP+batch, OTLP-no-batch). It takes\n~24 min in nightly runs (e.g. [run\n25835389073](https://github.com/open-telemetry/otel-arrow/actions/runs/25835389073/job/75909238633)).\n\nThe no-batch baseline variants (`otap-otap`, `otlp-otlp`) don't actually\nexercise the batch processor, and equivalent passthrough OTAP/OTLP\ncoverage already exists in other nightly suites (`otelcol-docker.yaml`,\n`100klrps-batch-sizes-docker.yaml`).\n\n## Change\n\nRemove the 6 no-batch baseline tests from this suite, keeping only the 6\nbatch-enabled variants. Expected wall-time saving: roughly half (~12\nmin).\n\n## Remaining tests\n\n- Logs-OTAP-BATCH-OTAP, Logs-OTLP-BATCH-OTLP\n- Metrics-OTAP-BATCH-OTAP, Metrics-OTLP-BATCH-OTLP\n- Traces-OTAP-BATCH-OTAP, Traces-OTLP-BATCH-OTLP\n\nAll batch_processor regression coverage across (signal × transport) is\npreserved.",
          "timestamp": "2026-05-14T15:35:01Z",
          "tree_id": "f19aa07db63bfe8dcbbfeb350155d2ba00354498",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/db9062212d3b6f4de5f72e1b1c875ed24b85f81b"
        },
        "date": 1778783488907,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.04709576070308685,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.0957393810556,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 99.77360173428306,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 31.573046875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 32.234375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 543553.5479841867,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 543297.5573022475,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002184,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 15460964.452846544,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 15410933.122611875,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
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
          "distinct": false,
          "id": "54598c7d91f10b5258113ceeb5f65baee0045cc9",
          "message": "Validate otel_* event names at compile time (#2957)\n\nAdds a `const fn validate_event_name` invoked from each `otel_info!` /\n`otel_warn!` / `otel_debug!` / `otel_error!` / `otel_event!` macro arm,\nso calls with empty or whitespace-containing event names fail `cargo\ncheck` instead of silently producing logs whose entire description is\nshoved into `event.name`.\n\nZero runtime cost (compile-time only). Existing call sites that violated\nthe rule are updated to use a short identifier with the descriptive text\nmoved to a `message = \"...\"` field.\n\nThe local macro copies in the `quiver` crate are intentionally left for\na follow-up.",
          "timestamp": "2026-05-14T15:40:17Z",
          "tree_id": "1eba98f7b02b4f1f65f1462a094f14855a2d3f95",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/54598c7d91f10b5258113ceeb5f65baee0045cc9"
        },
        "date": 1778785028671,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.0861191600561142,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.0003003291857,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.03549918799783,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 32.442708333333336,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 33.34375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 544952.1329815411,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 545421.4411677774,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003215,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 15487343.691512067,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 15437512.335388213,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
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
          "id": "300c8733c5e7430472ace73b6e92cdccded66294",
          "message": "chore(deps): update pipeline perf python dependencies (#2931)\n\nThis PR contains the following updates:\n\n| Package | Change |\n[Age](https://docs.renovatebot.com/merge-confidence/) |\n[Confidence](https://docs.renovatebot.com/merge-confidence/) |\n|---|---|---|---|\n| [pandas](https://redirect.github.com/pandas-dev/pandas) | `==3.0.2` →\n`==3.0.3` |\n![age](https://developer.mend.io/api/mc/badges/age/pypi/pandas/3.0.3?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/pypi/pandas/3.0.2/3.0.3?slim=true)\n|\n| [requests](https://redirect.github.com/psf/requests)\n([changelog](https://redirect.github.com/psf/requests/blob/master/HISTORY.md))\n| `==2.33.1` → `==2.34.1` |\n![age](https://developer.mend.io/api/mc/badges/age/pypi/requests/2.34.1?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/pypi/requests/2.33.1/2.34.1?slim=true)\n|\n\n---\n\n### Release Notes\n\n<details>\n<summary>pandas-dev/pandas (pandas)</summary>\n\n###\n[`v3.0.3`](https://redirect.github.com/pandas-dev/pandas/releases/tag/v3.0.3):\npandas 3.0.3\n\n[Compare\nSource](https://redirect.github.com/pandas-dev/pandas/compare/v3.0.2...v3.0.3)\n\nWe are pleased to announce the release of pandas 3.0.3.\nThis is a patch release in the 3.0.x series and includes some regression\nfixes and bug fixes. We recommend that all users of the 3.0.x series\nupgrade to this version.\n\nSee the [full\nwhatsnew](https://pandas.pydata.org/docs/whatsnew/v3.0.3.html) for a\nlist of all the changes.\n\nPandas 3.0 supports Python 3.11 and higher.\nThe release can be installed from PyPI:\n\n```\npython -m pip install --upgrade pandas==3.0.*\n```\n\nOr from conda-forge\n\n```\nconda install -c conda-forge pandas=3.0\n```\n\nPlease report any issues with the release on the [pandas issue\ntracker](https://redirect.github.com/pandas-dev/pandas/issues).\n\nThanks to all the contributors who made this release possible.\n\n</details>\n\n<details>\n<summary>psf/requests (requests)</summary>\n\n###\n[`v2.34.1`](https://redirect.github.com/psf/requests/blob/HEAD/HISTORY.md#2341-2026-05-13)\n\n[Compare\nSource](https://redirect.github.com/psf/requests/compare/v2.34.0...v2.34.1)\n\n**Bugfixes**\n\n- Widened `json` input type from `dict` and `list` to `Mapping`\nand `Sequence`.\n([#&#8203;7436](https://redirect.github.com/psf/requests/issues/7436))\n- Changed `headers` input type to MutableMapping and removed `None` from\n`Request.headers` typing to improve handling for users.\n([#&#8203;7431](https://redirect.github.com/psf/requests/issues/7431))\n- `Response.reason` moved from `str | None` to `str` to improve handling\nfor users.\n([#&#8203;7437](https://redirect.github.com/psf/requests/issues/7437))\n- Fixed a bug where some bodies with custom `__getattr__`\nimplementations\nweren't being properly detected as Iterables.\n([#&#8203;7433](https://redirect.github.com/psf/requests/issues/7433))\n\n###\n[`v2.34.0`](https://redirect.github.com/psf/requests/blob/HEAD/HISTORY.md#2340-2026-05-11)\n\n[Compare\nSource](https://redirect.github.com/psf/requests/compare/v2.33.1...v2.34.0)\n\n**Announcements**\n\n- Requests 2.34.0 introduces inline types, replacing those provided by\ntypeshed. Public API types should be fully compatible with mypy,\npyright,\nand ty. We believe types are comprehensive but if you find issues,\nplease\n  report them to the pinned tracking issue.\n\nSpecial thanks to\n[@&#8203;bastimeyer](https://redirect.github.com/bastimeyer),\n[@&#8203;cthoyt](https://redirect.github.com/cthoyt),\n[@&#8203;edgarrmondragon](https://redirect.github.com/edgarrmondragon),\nand [@&#8203;srittau](https://redirect.github.com/srittau) for\nhelping review and test the types ahead of the release.\n([#&#8203;7272](https://redirect.github.com/psf/requests/issues/7272))\n\n**Improvements**\n\n- Digest Auth hashing algorithms have added `usedforsecurity=False` to\nclarify\nsecurity considerations.\n([#&#8203;7310](https://redirect.github.com/psf/requests/issues/7310))\n- Requests added support for Python 3.15 based on beta1. Downstream\nprojects\nshould be able to start testing prior to its release in October.\n([#&#8203;7422](https://redirect.github.com/psf/requests/issues/7422))\n- Requests added support for Python 3.14t.\n([#&#8203;7419](https://redirect.github.com/psf/requests/issues/7419))\n\n**Bugfixes**\n\n- `Response.history` no longer contains a reference to itself,\npreventing\naccidental looping when traversing the history list.\n([#&#8203;7328](https://redirect.github.com/psf/requests/issues/7328))\n- Requests no longer performs greedy matching on no\\_proxy domains. The\n  proxy\\_bypass implementation has been updated with CPython's fix from\nbpo-39057.\n([#&#8203;7427](https://redirect.github.com/psf/requests/issues/7427))\n- Requests no longer incorrectly strips duplicate leading slashes in\n  URI paths. This should address user issues with specific presigned\nURLs. Note the full fix requires urllib3 2.7.0+.\n([#&#8203;7315](https://redirect.github.com/psf/requests/issues/7315))\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: (UTC)\n\n- Branch creation\n  - \"before 8am every weekday\"\n- Automerge\n  - At any time (no schedule defined)\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n👻 **Immortal**: This PR will be recreated if closed unmerged. Get\n[config\nhelp](https://redirect.github.com/renovatebot/renovate/discussions) if\nthat's undesired.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My4xNTkuMiIsInVwZGF0ZWRJblZlciI6IjQzLjE3My42IiwidGFyZ2V0QnJhbmNoIjoibWFpbiIsImxhYmVscyI6WyJkZXBlbmRlbmNpZXMiXX0=-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2026-05-14T16:02:07Z",
          "tree_id": "121dc728c32272834700c5e7ecb96eef320d787a",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/300c8733c5e7430472ace73b6e92cdccded66294"
        },
        "date": 1778788639786,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.7816817760467529,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.20371751219731,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.36092677256877,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 32.658203125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 33.20703125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 540343.5874188459,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 544567.354900446,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003303,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 15474664.033396948,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 15426755.49057537,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
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
          "id": "40b4f4d1112b1bc55f08185aa778865b4a43bd66",
          "message": "Set cache-bin: false on Swatinem/rust-cache to fix broken cargo on macos-latest (#2978)\n\n## Problem\n\nCI `clippy (*, macos-latest)` (and other macOS rust steps) started\nfailing today across many PRs with:\n\n```\nerror: error: unexpected argument 'clippy' found\nUsage: rustup-init[EXE] [OPTIONS]\n```\n\n## Root cause\n\nGitHub rolled out a new macos-latest runner image today\n([actions/runner-images#14037](https://github.com/actions/runner-images/pull/14037))\nthat changed how the `rustc`/`cargo` rustup proxy binaries are set up.\nCombined with\n[Swatinem/rust-cache#325](https://github.com/Swatinem/rust-cache/pull/325)\n(which made `cache-bin: true` the default in v2.8+), the cached\n`$CARGO_HOME/bin/` from previous runs gets restored over the\nfreshly-installed proxies, leaving `cargo` dispatching to `rustup-init`\nbehavior instead of the real cargo.\n\nTracked upstream:\n[Swatinem/rust-cache#341](https://github.com/Swatinem/rust-cache/issues/341).\n\n## Fix\n\nSet `cache-bin: false` on all 7 `Swatinem/rust-cache` invocations in\n`.github/workflows/rust-ci.yml`. This is the workaround confirmed by the\nupstream issue reporter. We don't `cargo install` any binaries that need\ncaching, so this loses no useful caching.",
          "timestamp": "2026-05-14T22:42:42Z",
          "tree_id": "165f39943b3c5c23dbec8d35f8c4e83ee60aa4b2",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/40b4f4d1112b1bc55f08185aa778865b4a43bd66"
        },
        "date": 1778801636860,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.49473848938941956,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.0284175809398,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.1754407670894,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 32.862239583333334,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 33.60546875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 543298.0281434,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 545985.9326331938,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002132,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 15548104.016322846,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 15498965.21565295,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "truffleagent@gmail.com",
            "name": "Truffle",
            "username": "truffle-dev"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "1bcb61866fbdc9b28420f409eb3de446fd8fcc02",
          "message": "Add OPL query-engine starts_with and ends_with functions (#2825)\n\nCloses #2819\n\nWires the upstream datafusion `starts_with` and `ends_with` UDFs into\nthe OPL query engine via the existing `InvokeFunctionExpr` path. Each\nfunction adds:\n\n- A function-name constant in `consts.rs`\n- A parser registration with two parameter placeholders in\n`parser.rs::default_parser_options`\n- A `from_func_name` arm in `DataFusionFunctionDef` (`expr.rs`)\nreturning `ExprLogicalType::Boolean` with `requires_dict_downcast:\ntrue`, matching the sha256 wiring\n\nExample queries that now work:\n\n```\nlogs | where starts_with(attributes[\"x\"], \"prefix\")\nlogs | where ends_with(event_name, \"suffix\")\n```\n\n## Tests\n\n- Unit tests in `expr.rs` build the `InvokeFunctionScalarExpression`\ndirectly, plan, execute against a `Logs` record batch, and assert a\n`BooleanArray` result. Patterned on `test_function_invocation_sha256`.\n- End-to-end OPL filter tests in `filter.rs` cover `event_name` and\n`attributes[\"...\"]` arguments, with the column on either side of the\npredicate.\n\n## Validation\n\n- `cargo check -p otap-df-query-engine`: clean\n- `cargo test -p otap-df-query-engine`: 548 passed (4 new filter tests,\n2 new expr tests)\n- `cargo clippy -p otap-df-query-engine --all-targets -- -D warnings`:\nclean\n- `cargo fmt --all -- --check`: clean\n- `cargo xtask quick-check`: clean\n\n## Notes\n\n`body` field tests are intentionally omitted because OTLP `body` is\nheterogeneous (`AnyValue` with string + int variants). The upstream\ndatafusion UDFs reject mixed types directly. `contains` works there\nbecause it has a custom string-coercing wrapper UDF; aligning\n`starts_with`/`ends_with` to that wrapper pattern is a follow-up beyond\nthe scope of #2819, which asks specifically for the upstream UDFs.\n\nSigned-off-by: truffle <truffleagent@gmail.com>\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-05-15T16:57:36Z",
          "tree_id": "a7924b87bf245f3e42bde72e5fb6737a6ab59980",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/1bcb61866fbdc9b28420f409eb3de446fd8fcc02"
        },
        "date": 1778868177167,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.6279927492141724,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.28495083889307,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.32110310635991,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 31.995182291666666,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 32.796875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 543499.8411673418,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 546912.9809187655,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003403,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 15538500.265274683,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 15489030.27095237,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
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
          "id": "672d665198917e2ede1ed856a9f88925ef2b151f",
          "message": "perf: add egress_bytes_per_log metric to benchmark reports (#2982)\n\n## Summary\n\nTwo changes to benchmark report metrics:\n\n### 1. Add `egress_bytes_per_log` metric\nAdds a derived metric (bytes/log) computed as `network_tx_bytes_rate /\nlogs_received_rate`. This makes it easy to assess whether compression\nratios in tests are realistic.\n\nFor a ~150 byte log record, realistic values should be ~35-50 bytes/log.\nValues like ~27 bytes/log indicate unrealistic compression from\nlow-entropy test data (e.g., replayed identical payloads).\n\n### 2. Switch network metrics from bytes/sec to MB/s\nReplaces `network_tx_bytes_rate_avg` and `network_rx_bytes_rate_avg`\n(bytes/sec) with `network_tx_mb_per_sec` and `network_rx_mb_per_sec`\n(MB/s) for readability. Raw bytes/sec values like 2,689,390 are hard to\ninterpret at a glance; 2.56 MB/s is immediately meaningful.\n\n### Files changed\n- `integration/configs/integration_report_logs.yaml`\n- `integration/configs/integration_report_metrics.yaml`\n- `integration/configs/integration_report_traces.yaml`\n- `comparison_dashboard/reports/report_logs.yaml`\n- `comparison_dashboard/reports/report_metrics.yaml`\n- `comparison_dashboard/reports/report_traces.yaml`\n\nRelates to #2540",
          "timestamp": "2026-05-15T22:28:24Z",
          "tree_id": "7618a842ff3874f6b9ef7e64a70aab82e18392e6",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/672d665198917e2ede1ed856a9f88925ef2b151f"
        },
        "date": 1778887051306,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.6440542936325073,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.23439127975621,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.32779284833539,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 32.80234375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 33.5078125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 534594.2483186683,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 543383.2678840032,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002142,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_mb_per_sec",
            "value": 14.715021930014245,
            "unit": "MB/s",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_mb_per_sec",
            "value": 14.673850711119632,
            "unit": "MB/s",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 28.39582988149801,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "52658021+Paramesh324@users.noreply.github.com",
            "name": "Paramesh324",
            "username": "Paramesh324"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "cf5c5783c4a9013cde8f9bd072b6df9b5c7b924e",
          "message": "Add coalesce support in OPL parser and OTAP query-engine planner (#2905)\n\n# Change Summary\n\nAdd support for `coalesce(...)` in OPL / OTAP query-engine by:\n- Parsing `coalesce` calls in the OPL parser into\n`ScalarExpression::Coalesce(CoalesceScalarExpression)`.\n- Planning `ScalarExpression::Coalesce` in the columnar query-engine\nexpression planner.\n- Lowering coalesce arguments to a `CASE/WHEN` logical expression\n(DataFusion-compatible behavior) to ensure correct execution in the\ncurrent planning/execution path.\n- Adding parser and planner/execution tests for valid usage and arity\nvalidation.\n\n## What issue does this PR close?\n\n* Closes #2823\n\n## How are these changes tested?\n\n- Added OPL parser tests for:\n  - valid `coalesce(...)` parsing\n  - invalid arity (fewer than 2 args)\n- Added query-engine expr planner/execution test for coalesce fallback\nbehavior.\n- Ran formatting and checks:\n  - `cargo fmt --all`\n  - `cargo test -p otap-df-opl --lib`\n  - `cargo test -p otap-df-query-engine --lib pipeline::expr::test`\n- `cargo clippy -p otap-df-opl -p otap-df-query-engine --all-targets --\n-D warnings`\n\n\n## Are there any user-facing changes?\n\n Yes.\nUsers can now use `coalesce(...)` in OPL expressions for OTAP\nquery-engine pipelines, e.g.:\n`coalesce(attributes[\"x\"], attributes[\"y\"], \"hello\")`\n\n---------\n\nSigned-off-by: Parameshwaran Krishnasamy <Parameshwaran.K@ibm.com>\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-05-18T13:59:55Z",
          "tree_id": "4a35a5e8c30142b698c7752cb6220d6581c162d4",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/cf5c5783c4a9013cde8f9bd072b6df9b5c7b924e"
        },
        "date": 1779118186841,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.3732337951660156,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.02272937829981,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.33478908188587,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 32.774609375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 33.72265625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 537494.8868548304,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 544875.9485650928,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002208,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_mb_per_sec",
            "value": 14.721821794183567,
            "unit": "MB/s",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_mb_per_sec",
            "value": 14.680436281673602,
            "unit": "MB/s",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 28.331125736620173,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          }
        ]
      }
    ]
  }
}