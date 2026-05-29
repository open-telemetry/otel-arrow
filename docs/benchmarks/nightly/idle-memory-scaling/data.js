window.BENCHMARK_DATA = {
  "lastUpdate": 1780079742127,
  "repoUrl": "https://github.com/open-telemetry/otel-arrow",
  "entries": {
    "Benchmark": [
      {
        "commit": {
          "author": {
            "name": "Cijo Thomas",
            "username": "cijothomas",
            "email": "cijo.thomas@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "2d53d89e89d477a39c38d39d863e942167f122ea",
          "message": "docs: document saturation test workload characteristics (#3021)\n\nDocument that saturation tests use static 1KB log bodies with realistic\nentropy (512 unique bodies), distinguishing them from other tests that\nuse semantic_conventions (~300 byte logs). Also removes the stale TODO\nand adds scaling efficiency formula explanation with link to the\nscaling-efficiency benchmark page.",
          "timestamp": "2026-05-18T23:28:48Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/2d53d89e89d477a39c38d39d863e942167f122ea"
        },
        "date": 1779157122764,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_memory_constant_overhead_mib",
            "value": 15.23,
            "unit": "MiB",
            "extra": "Constant memory overhead (C in Memory = C + N*R)"
          },
          {
            "name": "idle_memory_per_core_overhead_mib",
            "value": 0.81,
            "unit": "MiB",
            "extra": "Per-core memory overhead (R in Memory = C + N*R)"
          },
          {
            "name": "idle_memory_r_squared",
            "value": 0.993,
            "unit": "",
            "extra": "Linear fit quality (R²); 1.0 = perfect linear scaling"
          },
          {
            "name": "idle_memory_1core_mib",
            "value": 14.99,
            "unit": "MiB",
            "extra": "Idle memory at 1 core(s); predicted=16.0 MiB, error=7.0%"
          },
          {
            "name": "idle_memory_2core_mib",
            "value": 17.29,
            "unit": "MiB",
            "extra": "Idle memory at 2 core(s); predicted=16.8 MiB, error=2.5%"
          },
          {
            "name": "idle_memory_4core_mib",
            "value": 18.24,
            "unit": "MiB",
            "extra": "Idle memory at 4 core(s); predicted=18.5 MiB, error=1.2%"
          },
          {
            "name": "idle_memory_8core_mib",
            "value": 21.98,
            "unit": "MiB",
            "extra": "Idle memory at 8 core(s); predicted=21.7 MiB, error=1.3%"
          },
          {
            "name": "idle_memory_16core_mib",
            "value": 29.35,
            "unit": "MiB",
            "extra": "Idle memory at 16 core(s); predicted=28.2 MiB, error=4.0%"
          },
          {
            "name": "idle_memory_32core_mib",
            "value": 40.47,
            "unit": "MiB",
            "extra": "Idle memory at 32 core(s); predicted=41.1 MiB, error=1.6%"
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
          "id": "39c8b738e5c0b91fb4c4b747d129b2847b4921f7",
          "message": "fix(deps): update module go.opentelemetry.io/collector/pdata to v1.58.0 - abandoned (#2999)\n\nThis PR contains the following updates:\n\n| Package | Change |\n[Age](https://docs.renovatebot.com/merge-confidence/) |\n[Confidence](https://docs.renovatebot.com/merge-confidence/) |\n|---|---|---|---|\n|\n[go.opentelemetry.io/collector/pdata](https://redirect.github.com/open-telemetry/opentelemetry-collector)\n| `v1.57.0` → `v1.58.0` |\n![age](https://developer.mend.io/api/mc/badges/age/go/go.opentelemetry.io%2fcollector%2fpdata/v1.58.0?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/go/go.opentelemetry.io%2fcollector%2fpdata/v1.57.0/v1.58.0?slim=true)\n|\n\n---\n\n### Release Notes\n\n<details>\n<summary>open-telemetry/opentelemetry-collector\n(go.opentelemetry.io/collector/pdata)</summary>\n\n###\n[`v1.58.0`](https://redirect.github.com/open-telemetry/opentelemetry-collector/blob/HEAD/CHANGELOG.md#v1580v01520)\n\n##### 💡 Enhancements 💡\n\n- `pkg/exporterhelper`: Add `otelcol_exporter_in_flight_requests` metric\nto track the number of export requests currently in-flight per exporter.\n([#&#8203;15009](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/15009))\nThis UpDownCounter increments in startOp and decrements in endOp,\nallowing operators to monitor\nconcurrent export activity and detect when an exporter is saturating its\nworker pool.\n\n##### 🧰 Bug fixes 🧰\n\n- `pkg/confighttp`: Close the original request body after reading\nblock-format `Content-Encoding: snappy` requests.\n([#&#8203;15262](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/15262))\n\n- `pkg/confighttp`: Recover from panics in decompression libraries,\nreturn HTTP 400 instead of 500.\n([#&#8203;13228](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/13228))\n\n- `pkg/confighttp`: Enforce `max_request_body_size` on\n`Content-Encoding: snappy` requests before the decoded buffer is\nallocated.\n([#&#8203;15252](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/15252))\n\n- `pkg/otelcol`: Stop emitting verbose gRPC transport messages at WARN\nduring normal client disconnect.\n([#&#8203;5169](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/5169))\ngrpc-go gates chatty per-RPC notices (e.g. \"HandleStreams failed to read\nframe:\nconnection reset by peer\") behind `LoggerV2.V(2)`. zapgrpc.Logger.V\nconflates\ngrpclog verbosity with zap severity, so V(2) returns true whenever WARN\nis\nenabled and these messages emit at WARN. Wrap the installed\ngrpclog.LoggerV2\nwith a corrected V() that compares against a fixed verbosity threshold,\nmatching grpclog's intended semantics. See\n[uber-go/zap#1544](https://redirect.github.com/uber-go/zap/issues/1544).\n\n- `pkg/pdata`: `pcommon.Value.AsString` no longer HTML-escapes `<`, `>`,\nand `&` inside `ValueTypeMap` and `ValueTypeSlice` values, matching the\nbehavior already used for `ValueTypeStr`.\n([#&#8203;14662](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/14662))\n\n- `pkg/service`: Fix Prometheus config defaults mismatch when host is\nexplicitly set in telemetry configuration.\n([#&#8203;13867](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/13867))\nWhen users explicitly configured the telemetry metrics section (e.g. to\nchange the host),\nthe Prometheus exporter boolean fields (WithoutScopeInfo, WithoutUnits,\nWithoutTypeSuffix)\ndefaulted to nil/false instead of true, causing metric name format\nchanges compared to the\nimplicit default configuration. This fix applies the correct defaults\nduring config unmarshaling.\n\n- `pkg/service`: Return noop tracer provider when no trace processors\nare defined\n([#&#8203;15135](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/15135))\n\n<!-- previous-version -->\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: (UTC)\n\n- Branch creation\n  - \"before 8am on Monday\"\n- Automerge\n  - At any time (no schedule defined)\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My4xNzkuMyIsInVwZGF0ZWRJblZlciI6IjQzLjE3OS4zIiwidGFyZ2V0QnJhbmNoIjoibWFpbiIsImxhYmVscyI6WyJkZXBlbmRlbmNpZXMiXX0=-->\n\n---------\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: otelbot <197425009+otelbot@users.noreply.github.com>",
          "timestamp": "2026-05-19T16:55:41Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/39c8b738e5c0b91fb4c4b747d129b2847b4921f7"
        },
        "date": 1779216283555,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_memory_constant_overhead_mib",
            "value": 15.17,
            "unit": "MiB",
            "extra": "Constant memory overhead (C in Memory = C + N*R)"
          },
          {
            "name": "idle_memory_per_core_overhead_mib",
            "value": 0.79,
            "unit": "MiB",
            "extra": "Per-core memory overhead (R in Memory = C + N*R)"
          },
          {
            "name": "idle_memory_r_squared",
            "value": 0.9971,
            "unit": "",
            "extra": "Linear fit quality (R²); 1.0 = perfect linear scaling"
          },
          {
            "name": "idle_memory_1core_mib",
            "value": 16.5,
            "unit": "MiB",
            "extra": "Idle memory at 1 core(s); predicted=16.0 MiB, error=3.3%"
          },
          {
            "name": "idle_memory_2core_mib",
            "value": 15.93,
            "unit": "MiB",
            "extra": "Idle memory at 2 core(s); predicted=16.8 MiB, error=5.2%"
          },
          {
            "name": "idle_memory_4core_mib",
            "value": 18.34,
            "unit": "MiB",
            "extra": "Idle memory at 4 core(s); predicted=18.3 MiB, error=0.1%"
          },
          {
            "name": "idle_memory_8core_mib",
            "value": 21.46,
            "unit": "MiB",
            "extra": "Idle memory at 8 core(s); predicted=21.5 MiB, error=0.1%"
          },
          {
            "name": "idle_memory_16core_mib",
            "value": 28.31,
            "unit": "MiB",
            "extra": "Idle memory at 16 core(s); predicted=27.8 MiB, error=1.8%"
          },
          {
            "name": "idle_memory_32core_mib",
            "value": 40.2,
            "unit": "MiB",
            "extra": "Idle memory at 32 core(s); predicted=40.4 MiB, error=0.5%"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Gokhan Uslu",
            "username": "gouslu",
            "email": "geukhanuslu@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "60f251825b8a2022b3c3373761eb6b55f9a30da0",
          "message": "feat(engine): wire extensions and capabilities into runtime pipeline (#2860)\n\n# Change Summary\n \n Part 4 of the Extension System (P1) series. Wires the previously\n landed Capability Registry & Resolver (#2732) into the runtime\n pipeline so extensions are actually instantiated, started, and\n shut down by the engine, and so consumer nodes can resolve their\n capability bindings at build time.\n \n Highlights:\n \n - **Runtime wiring** in `runtime_pipeline.rs`: extension lifecycle\n   is invoked before any data-path node is spawned, and `Shutdown`\n   is delivered to extensions only after the data path drains\n   (\"started first, shut down last\"). Active and passive extensions\n   are handled separately; failures abort startup cleanly.\n - **Local capability ownership aligned with shared** via a\n   Box-clone factory pattern, removing the prior asymmetry between\n   the two trait variants.\n - **Two reference test capabilities** under\n   `crates/engine/src/testing/capability/`: `NoOpStateless` and\n   `NoOpStateful`. They exercise every codegen path of the\n   `#[capability]` proc macro (`&self` × {sync, async}, `&mut self`\n   × {sync, async}, borrowed/owned returns, etc.). Test-only; they\n   intentionally live under `testing/` rather than the public\n   `capability/` surface.\n - **Comprehensive end-to-end test suite** at\n   `crates/engine/tests/extension_e2e.rs` (26 tests) covering:\n   passive/active/background extensions, lifecycle ordering and\n   shutdown ordering, fail-fast on extension errors, dual-variant\n   pruning, one-shot capability enforcement (all accessor\n   combinations), shared mutable state across consumers via\n   `Arc`/`Rc` for both local and shared trait variants, async\n   `&mut self` invocation through boxed handles, and active\n   extensions mutating shared state observed by capability\n   consumers.\n - **Architecture doc** updated with a precise statement of the\n   start-first/shut-down-last invariant (it orders lifecycle\n   *calls*, not init completion) and a noted future consideration\n   to add an opt-in readiness probe if/when an extension needs an\n   init-complete guarantee.\n - **URN unification**: extension URNs now use the canonical\n   4-segment form `urn:<namespace>:extension:<id>` (mirroring the\n   receiver/processor/exporter convention), with a short form\n   `extension:<id>`. The shared parser core lives in a new\n   private `crates/config/src/urn.rs`; `node_urn.rs` and\n   `extension_urn.rs` delegate to it with disjoint accepted-kind\n   sets so the two URN types cannot be confused. As a consequence,\n   `NodeKind::Extension` and the now-unreachable\n   `Error::ExtensionInNodesSection` are removed. Misplacement\n   errors include actionable hints (e.g. *\"declare under\n   `extensions:` instead of `nodes:`\"*).\n - All in-tree node factories (receivers, processors, exporters\n   in `core-nodes` and `contrib-nodes`) updated to accept the new\n   `&Capabilities` parameter; existing factories that don't depend\n   on any capability simply ignore it.\n \n ## What issue does this PR close?\n \n ## How are these changes tested?\n \n - New `extension_e2e.rs` integration test (26 tests) exercises the\n   wiring end-to-end against synthetic receivers/processors/\n   exporters/extensions.\n - New unit tests in `urn.rs` cover the shared parser core and the\n   misplacement-error hints; existing `extension_urn` and\n   `node_urn` tests updated to assert the canonical 4-segment form.\n - Pipeline-level regression tests cover rejecting extension URNs\n   in the `nodes:` section and node URNs in the `extensions:`\n   section.\n - `cargo xtask check` (structure check + `fmt` + `clippy --workspace\n   --all-targets -- -D warnings` + `cargo test --workspace`) passes\n   cleanly. No new clippy warnings.\n \n ## Are there any user-facing changes?\n \n Yes:\n \n - **Extension URN format**: extension URNs now use\n   `urn:<namespace>:extension:<id>` (4-segment) instead of the\n   pre-existing 3-segment `urn:<namespace>:<id>`. Short form\n   `extension:<id>` (expands to `urn:otel:extension:<id>`) is\n   available as a developer convenience. Existing 3-segment\n   extension URNs in pipeline configs must be updated. The\n   previously-bundled `configs/fake-with-extension.yaml` was an\n   orphan (its URN had no registered `ExtensionFactory` anywhere\n   in the binary, and it had no test/script/doc consumers) and\n   was removed in `482feb22c`; the canonical 4-segment shape is\n   covered by the `test_extension_with_config_and_capabilities`\n   unit test in `crates/config/src/pipeline.rs`. A runnable demo\n   config can land in a follow-up alongside a real factory.\n - **New extension authoring surface**: `Extension` trait,\n   `ExtensionWrapper::builder` typestate, the\n   `extension_capabilities!` macro, and the test capabilities\n   `NoOpStateless` / `NoOpStateful` (under `testing/capability/`)\n   are now reachable for external extension authors. The\n   architecture doc captures the lifecycle contract.\n - **Node factory signature** now includes `&Capabilities` as a\n   parameter; existing custom factories will need to accept (and\n   may ignore) this new argument\n\n---------\n\nCo-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>",
          "timestamp": "2026-05-19T23:25:05Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/60f251825b8a2022b3c3373761eb6b55f9a30da0"
        },
        "date": 1779245078255,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_memory_constant_overhead_mib",
            "value": 15.11,
            "unit": "MiB",
            "extra": "Constant memory overhead (C in Memory = C + N*R)"
          },
          {
            "name": "idle_memory_per_core_overhead_mib",
            "value": 0.84,
            "unit": "MiB",
            "extra": "Per-core memory overhead (R in Memory = C + N*R)"
          },
          {
            "name": "idle_memory_r_squared",
            "value": 0.9987,
            "unit": "",
            "extra": "Linear fit quality (R²); 1.0 = perfect linear scaling"
          },
          {
            "name": "idle_memory_1core_mib",
            "value": 15.76,
            "unit": "MiB",
            "extra": "Idle memory at 1 core(s); predicted=15.9 MiB, error=1.2%"
          },
          {
            "name": "idle_memory_2core_mib",
            "value": 16.99,
            "unit": "MiB",
            "extra": "Idle memory at 2 core(s); predicted=16.8 MiB, error=1.2%"
          },
          {
            "name": "idle_memory_4core_mib",
            "value": 18.49,
            "unit": "MiB",
            "extra": "Idle memory at 4 core(s); predicted=18.5 MiB, error=0.1%"
          },
          {
            "name": "idle_memory_8core_mib",
            "value": 21.38,
            "unit": "MiB",
            "extra": "Idle memory at 8 core(s); predicted=21.8 MiB, error=2.1%"
          },
          {
            "name": "idle_memory_16core_mib",
            "value": 29.13,
            "unit": "MiB",
            "extra": "Idle memory at 16 core(s); predicted=28.5 MiB, error=2.0%"
          },
          {
            "name": "idle_memory_32core_mib",
            "value": 41.76,
            "unit": "MiB",
            "extra": "Idle memory at 32 core(s); predicted=42.0 MiB, error=0.5%"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Jake Dern",
            "username": "JakeDern",
            "email": "33842784+JakeDern@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "fabbe70cbf95113006d75b3e89725cd930d747c3",
          "message": "task(comparison_dashboard): Update the banner (#3050)\n\n# Change Summary\n\nRather than remove, I thought we might want to update the banner with\nsome new text and a link to file issues for feedback.\n\nI know the name \"Dataflow Engine\" is up for some debate, though we\nalready use this name elsewhere in the site.\n\nOpen to suggestions on all fronts including just removing the banner!\n\n## What issue does this PR close?\n\n* Closes #3019\n\n## How are these changes tested?\n\n<img width=\"2435\" height=\"817\" alt=\"image\"\nsrc=\"https://github.com/user-attachments/assets/b3cb0454-37b5-4afd-81f0-7dc6acac0136\"\n/>\n\n## Are there any user-facing changes?\n\nYes - Banner update.\n\nCo-authored-by: Cijo Thomas <cijo.thomas@gmail.com>",
          "timestamp": "2026-05-20T16:17:47Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/fabbe70cbf95113006d75b3e89725cd930d747c3"
        },
        "date": 1779303398052,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_memory_constant_overhead_mib",
            "value": 15.81,
            "unit": "MiB",
            "extra": "Constant memory overhead (C in Memory = C + N*R)"
          },
          {
            "name": "idle_memory_per_core_overhead_mib",
            "value": 0.77,
            "unit": "MiB",
            "extra": "Per-core memory overhead (R in Memory = C + N*R)"
          },
          {
            "name": "idle_memory_r_squared",
            "value": 0.9974,
            "unit": "",
            "extra": "Linear fit quality (R²); 1.0 = perfect linear scaling"
          },
          {
            "name": "idle_memory_1core_mib",
            "value": 16.38,
            "unit": "MiB",
            "extra": "Idle memory at 1 core(s); predicted=16.6 MiB, error=1.2%"
          },
          {
            "name": "idle_memory_2core_mib",
            "value": 17.02,
            "unit": "MiB",
            "extra": "Idle memory at 2 core(s); predicted=17.3 MiB, error=1.9%"
          },
          {
            "name": "idle_memory_4core_mib",
            "value": 18.91,
            "unit": "MiB",
            "extra": "Idle memory at 4 core(s); predicted=18.9 MiB, error=0.1%"
          },
          {
            "name": "idle_memory_8core_mib",
            "value": 22.05,
            "unit": "MiB",
            "extra": "Idle memory at 8 core(s); predicted=22.0 MiB, error=0.3%"
          },
          {
            "name": "idle_memory_16core_mib",
            "value": 29.02,
            "unit": "MiB",
            "extra": "Idle memory at 16 core(s); predicted=28.2 MiB, error=3.0%"
          },
          {
            "name": "idle_memory_32core_mib",
            "value": 40.07,
            "unit": "MiB",
            "extra": "Idle memory at 32 core(s); predicted=40.5 MiB, error=1.1%"
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
          "id": "84dfec92b99db44ed2e74a980b1f40df5f4b3ee9",
          "message": "Update one_collect digest to 6ccba44 (#2979)\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n|---|---|---|---|\n| one_collect | workspace.dependencies | digest | `cfe3f78` → `6ccba44`\n|\n\n---\n\n### Configuration\n\n📅 **Schedule**: (UTC)\n\n- Branch creation\n  - At any time (no schedule defined)\n- Automerge\n  - At any time (no schedule defined)\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My4xNzkuMyIsInVwZGF0ZWRJblZlciI6IjQzLjE4NS4xIiwidGFyZ2V0QnJhbmNoIjoibWFpbiIsImxhYmVscyI6WyJkZXBlbmRlbmNpZXMiXX0=-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2026-05-20T22:51:33Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/84dfec92b99db44ed2e74a980b1f40df5f4b3ee9"
        },
        "date": 1779331417400,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_memory_constant_overhead_mib",
            "value": 14.47,
            "unit": "MiB",
            "extra": "Constant memory overhead (C in Memory = C + N*R)"
          },
          {
            "name": "idle_memory_per_core_overhead_mib",
            "value": 0.83,
            "unit": "MiB",
            "extra": "Per-core memory overhead (R in Memory = C + N*R)"
          },
          {
            "name": "idle_memory_r_squared",
            "value": 0.9985,
            "unit": "",
            "extra": "Linear fit quality (R²); 1.0 = perfect linear scaling"
          },
          {
            "name": "idle_memory_1core_mib",
            "value": 15.57,
            "unit": "MiB",
            "extra": "Idle memory at 1 core(s); predicted=15.3 MiB, error=1.8%"
          },
          {
            "name": "idle_memory_2core_mib",
            "value": 16.15,
            "unit": "MiB",
            "extra": "Idle memory at 2 core(s); predicted=16.1 MiB, error=0.2%"
          },
          {
            "name": "idle_memory_4core_mib",
            "value": 17.1,
            "unit": "MiB",
            "extra": "Idle memory at 4 core(s); predicted=17.8 MiB, error=3.9%"
          },
          {
            "name": "idle_memory_8core_mib",
            "value": 21.21,
            "unit": "MiB",
            "extra": "Idle memory at 8 core(s); predicted=21.1 MiB, error=0.6%"
          },
          {
            "name": "idle_memory_16core_mib",
            "value": 28.08,
            "unit": "MiB",
            "extra": "Idle memory at 16 core(s); predicted=27.7 MiB, error=1.4%"
          },
          {
            "name": "idle_memory_32core_mib",
            "value": 40.77,
            "unit": "MiB",
            "extra": "Idle memory at 32 core(s); predicted=40.9 MiB, error=0.4%"
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
          "id": "32abb25dd613ea36a97504ba79da5e427a3bef72",
          "message": "Add AaronRM as Triager (#3063)\n\n# Change Summary\n\nUpdate docs\n\n## What issue does this PR close?\n\n* Closes #3062",
          "timestamp": "2026-05-22T15:56:41Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/32abb25dd613ea36a97504ba79da5e427a3bef72"
        },
        "date": 1779474346640,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_memory_constant_overhead_mib",
            "value": 15.61,
            "unit": "MiB",
            "extra": "Constant memory overhead (C in Memory = C + N*R)"
          },
          {
            "name": "idle_memory_per_core_overhead_mib",
            "value": 0.74,
            "unit": "MiB",
            "extra": "Per-core memory overhead (R in Memory = C + N*R)"
          },
          {
            "name": "idle_memory_r_squared",
            "value": 0.9894,
            "unit": "",
            "extra": "Linear fit quality (R²); 1.0 = perfect linear scaling"
          },
          {
            "name": "idle_memory_1core_mib",
            "value": 16.35,
            "unit": "MiB",
            "extra": "Idle memory at 1 core(s); predicted=16.4 MiB, error=0.0%"
          },
          {
            "name": "idle_memory_2core_mib",
            "value": 16.41,
            "unit": "MiB",
            "extra": "Idle memory at 2 core(s); predicted=17.1 MiB, error=4.1%"
          },
          {
            "name": "idle_memory_4core_mib",
            "value": 18.41,
            "unit": "MiB",
            "extra": "Idle memory at 4 core(s); predicted=18.6 MiB, error=0.9%"
          },
          {
            "name": "idle_memory_8core_mib",
            "value": 21.43,
            "unit": "MiB",
            "extra": "Idle memory at 8 core(s); predicted=21.6 MiB, error=0.5%"
          },
          {
            "name": "idle_memory_16core_mib",
            "value": 29.24,
            "unit": "MiB",
            "extra": "Idle memory at 16 core(s); predicted=27.5 MiB, error=6.0%"
          },
          {
            "name": "idle_memory_32core_mib",
            "value": 38.6,
            "unit": "MiB",
            "extra": "Idle memory at 32 core(s); predicted=39.4 MiB, error=2.0%"
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
          "id": "32abb25dd613ea36a97504ba79da5e427a3bef72",
          "message": "Add AaronRM as Triager (#3063)\n\n# Change Summary\n\nUpdate docs\n\n## What issue does this PR close?\n\n* Closes #3062",
          "timestamp": "2026-05-22T15:56:41Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/32abb25dd613ea36a97504ba79da5e427a3bef72"
        },
        "date": 1779507450829,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_memory_constant_overhead_mib",
            "value": 15.07,
            "unit": "MiB",
            "extra": "Constant memory overhead (C in Memory = C + N*R)"
          },
          {
            "name": "idle_memory_per_core_overhead_mib",
            "value": 0.83,
            "unit": "MiB",
            "extra": "Per-core memory overhead (R in Memory = C + N*R)"
          },
          {
            "name": "idle_memory_r_squared",
            "value": 0.9997,
            "unit": "",
            "extra": "Linear fit quality (R²); 1.0 = perfect linear scaling"
          },
          {
            "name": "idle_memory_1core_mib",
            "value": 15.76,
            "unit": "MiB",
            "extra": "Idle memory at 1 core(s); predicted=15.9 MiB, error=0.9%"
          },
          {
            "name": "idle_memory_2core_mib",
            "value": 16.52,
            "unit": "MiB",
            "extra": "Idle memory at 2 core(s); predicted=16.7 MiB, error=1.3%"
          },
          {
            "name": "idle_memory_4core_mib",
            "value": 18.63,
            "unit": "MiB",
            "extra": "Idle memory at 4 core(s); predicted=18.4 MiB, error=1.3%"
          },
          {
            "name": "idle_memory_8core_mib",
            "value": 21.86,
            "unit": "MiB",
            "extra": "Idle memory at 8 core(s); predicted=21.7 MiB, error=0.7%"
          },
          {
            "name": "idle_memory_16core_mib",
            "value": 28.35,
            "unit": "MiB",
            "extra": "Idle memory at 16 core(s); predicted=28.3 MiB, error=0.0%"
          },
          {
            "name": "idle_memory_32core_mib",
            "value": 41.58,
            "unit": "MiB",
            "extra": "Idle memory at 32 core(s); predicted=41.6 MiB, error=0.1%"
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
          "id": "32abb25dd613ea36a97504ba79da5e427a3bef72",
          "message": "Add AaronRM as Triager (#3063)\n\n# Change Summary\n\nUpdate docs\n\n## What issue does this PR close?\n\n* Closes #3062",
          "timestamp": "2026-05-22T15:56:41Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/32abb25dd613ea36a97504ba79da5e427a3bef72"
        },
        "date": 1779557614508,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_memory_constant_overhead_mib",
            "value": 14.69,
            "unit": "MiB",
            "extra": "Constant memory overhead (C in Memory = C + N*R)"
          },
          {
            "name": "idle_memory_per_core_overhead_mib",
            "value": 0.81,
            "unit": "MiB",
            "extra": "Per-core memory overhead (R in Memory = C + N*R)"
          },
          {
            "name": "idle_memory_r_squared",
            "value": 0.9996,
            "unit": "",
            "extra": "Linear fit quality (R²); 1.0 = perfect linear scaling"
          },
          {
            "name": "idle_memory_1core_mib",
            "value": 15.66,
            "unit": "MiB",
            "extra": "Idle memory at 1 core(s); predicted=15.5 MiB, error=1.0%"
          },
          {
            "name": "idle_memory_2core_mib",
            "value": 16.26,
            "unit": "MiB",
            "extra": "Idle memory at 2 core(s); predicted=16.3 MiB, error=0.4%"
          },
          {
            "name": "idle_memory_4core_mib",
            "value": 17.74,
            "unit": "MiB",
            "extra": "Idle memory at 4 core(s); predicted=17.9 MiB, error=1.1%"
          },
          {
            "name": "idle_memory_8core_mib",
            "value": 21.44,
            "unit": "MiB",
            "extra": "Idle memory at 8 core(s); predicted=21.2 MiB, error=1.1%"
          },
          {
            "name": "idle_memory_16core_mib",
            "value": 27.48,
            "unit": "MiB",
            "extra": "Idle memory at 16 core(s); predicted=27.7 MiB, error=0.8%"
          },
          {
            "name": "idle_memory_32core_mib",
            "value": 40.8,
            "unit": "MiB",
            "extra": "Idle memory at 32 core(s); predicted=40.7 MiB, error=0.2%"
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
          "id": "32abb25dd613ea36a97504ba79da5e427a3bef72",
          "message": "Add AaronRM as Triager (#3063)\n\n# Change Summary\n\nUpdate docs\n\n## What issue does this PR close?\n\n* Closes #3062",
          "timestamp": "2026-05-22T15:56:41Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/32abb25dd613ea36a97504ba79da5e427a3bef72"
        },
        "date": 1779589171874,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_memory_constant_overhead_mib",
            "value": 15.85,
            "unit": "MiB",
            "extra": "Constant memory overhead (C in Memory = C + N*R)"
          },
          {
            "name": "idle_memory_per_core_overhead_mib",
            "value": 0.76,
            "unit": "MiB",
            "extra": "Per-core memory overhead (R in Memory = C + N*R)"
          },
          {
            "name": "idle_memory_r_squared",
            "value": 0.9944,
            "unit": "",
            "extra": "Linear fit quality (R²); 1.0 = perfect linear scaling"
          },
          {
            "name": "idle_memory_1core_mib",
            "value": 15.48,
            "unit": "MiB",
            "extra": "Idle memory at 1 core(s); predicted=16.6 MiB, error=7.3%"
          },
          {
            "name": "idle_memory_2core_mib",
            "value": 17.45,
            "unit": "MiB",
            "extra": "Idle memory at 2 core(s); predicted=17.4 MiB, error=0.5%"
          },
          {
            "name": "idle_memory_4core_mib",
            "value": 19.16,
            "unit": "MiB",
            "extra": "Idle memory at 4 core(s); predicted=18.9 MiB, error=1.4%"
          },
          {
            "name": "idle_memory_8core_mib",
            "value": 22.81,
            "unit": "MiB",
            "extra": "Idle memory at 8 core(s); predicted=21.9 MiB, error=3.8%"
          },
          {
            "name": "idle_memory_16core_mib",
            "value": 28.3,
            "unit": "MiB",
            "extra": "Idle memory at 16 core(s); predicted=28.0 MiB, error=0.9%"
          },
          {
            "name": "idle_memory_32core_mib",
            "value": 39.89,
            "unit": "MiB",
            "extra": "Idle memory at 32 core(s); predicted=40.2 MiB, error=0.9%"
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
          "id": "32abb25dd613ea36a97504ba79da5e427a3bef72",
          "message": "Add AaronRM as Triager (#3063)\n\n# Change Summary\n\nUpdate docs\n\n## What issue does this PR close?\n\n* Closes #3062",
          "timestamp": "2026-05-22T15:56:41Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/32abb25dd613ea36a97504ba79da5e427a3bef72"
        },
        "date": 1779644045849,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_memory_constant_overhead_mib",
            "value": 15.06,
            "unit": "MiB",
            "extra": "Constant memory overhead (C in Memory = C + N*R)"
          },
          {
            "name": "idle_memory_per_core_overhead_mib",
            "value": 0.77,
            "unit": "MiB",
            "extra": "Per-core memory overhead (R in Memory = C + N*R)"
          },
          {
            "name": "idle_memory_r_squared",
            "value": 0.9992,
            "unit": "",
            "extra": "Linear fit quality (R²); 1.0 = perfect linear scaling"
          },
          {
            "name": "idle_memory_1core_mib",
            "value": 16.32,
            "unit": "MiB",
            "extra": "Idle memory at 1 core(s); predicted=15.8 MiB, error=3.0%"
          },
          {
            "name": "idle_memory_2core_mib",
            "value": 16.49,
            "unit": "MiB",
            "extra": "Idle memory at 2 core(s); predicted=16.6 MiB, error=0.8%"
          },
          {
            "name": "idle_memory_4core_mib",
            "value": 17.87,
            "unit": "MiB",
            "extra": "Idle memory at 4 core(s); predicted=18.2 MiB, error=1.6%"
          },
          {
            "name": "idle_memory_8core_mib",
            "value": 21.21,
            "unit": "MiB",
            "extra": "Idle memory at 8 core(s); predicted=21.3 MiB, error=0.3%"
          },
          {
            "name": "idle_memory_16core_mib",
            "value": 27.35,
            "unit": "MiB",
            "extra": "Idle memory at 16 core(s); predicted=27.5 MiB, error=0.4%"
          },
          {
            "name": "idle_memory_32core_mib",
            "value": 39.95,
            "unit": "MiB",
            "extra": "Idle memory at 32 core(s); predicted=39.9 MiB, error=0.2%"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Jake Dern",
            "username": "JakeDern",
            "email": "33842784+JakeDern@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "dae17d6eedbb715a81942cb8616a761ec45eeae3",
          "message": "perf: Box inner raw_batch_store record batch slices (#3077)\n\n# Change Summary\n\nBoxes raw_batch_store record batch slices. Attached issue explains the\nrationale, but basically, we have very a very large enum variant for\notap metrics (almost 1k) which is penalizing a lot of data structures.\n\nThis penalty is more visible when queues are larger and/or saturated,\nwhen batches are smaller in size, or when there are many processing\nstages.\n\n## What issue does this PR close?\n\n* Closes #3076\n\n## How are these changes tested?\n\nAd-hoc perf testing and also the manual pipelineperf run for the\nstandard continuous bench set.\n\n## Are there any user-facing changes?\n\nNo.",
          "timestamp": "2026-05-24T17:52:44Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/dae17d6eedbb715a81942cb8616a761ec45eeae3"
        },
        "date": 1779675703260,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_memory_constant_overhead_mib",
            "value": 15.24,
            "unit": "MiB",
            "extra": "Constant memory overhead (C in Memory = C + N*R)"
          },
          {
            "name": "idle_memory_per_core_overhead_mib",
            "value": 0.78,
            "unit": "MiB",
            "extra": "Per-core memory overhead (R in Memory = C + N*R)"
          },
          {
            "name": "idle_memory_r_squared",
            "value": 0.9922,
            "unit": "",
            "extra": "Linear fit quality (R²); 1.0 = perfect linear scaling"
          },
          {
            "name": "idle_memory_1core_mib",
            "value": 16.59,
            "unit": "MiB",
            "extra": "Idle memory at 1 core(s); predicted=16.0 MiB, error=3.4%"
          },
          {
            "name": "idle_memory_2core_mib",
            "value": 15.94,
            "unit": "MiB",
            "extra": "Idle memory at 2 core(s); predicted=16.8 MiB, error=5.4%"
          },
          {
            "name": "idle_memory_4core_mib",
            "value": 17.88,
            "unit": "MiB",
            "extra": "Idle memory at 4 core(s); predicted=18.4 MiB, error=2.6%"
          },
          {
            "name": "idle_memory_8core_mib",
            "value": 21.47,
            "unit": "MiB",
            "extra": "Idle memory at 8 core(s); predicted=21.5 MiB, error=0.0%"
          },
          {
            "name": "idle_memory_16core_mib",
            "value": 29,
            "unit": "MiB",
            "extra": "Idle memory at 16 core(s); predicted=27.7 MiB, error=4.6%"
          },
          {
            "name": "idle_memory_32core_mib",
            "value": 39.55,
            "unit": "MiB",
            "extra": "Idle memory at 32 core(s); predicted=40.1 MiB, error=1.4%"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Jake Dern",
            "username": "JakeDern",
            "email": "33842784+JakeDern@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "dae17d6eedbb715a81942cb8616a761ec45eeae3",
          "message": "perf: Box inner raw_batch_store record batch slices (#3077)\n\n# Change Summary\n\nBoxes raw_batch_store record batch slices. Attached issue explains the\nrationale, but basically, we have very a very large enum variant for\notap metrics (almost 1k) which is penalizing a lot of data structures.\n\nThis penalty is more visible when queues are larger and/or saturated,\nwhen batches are smaller in size, or when there are many processing\nstages.\n\n## What issue does this PR close?\n\n* Closes #3076\n\n## How are these changes tested?\n\nAd-hoc perf testing and also the manual pipelineperf run for the\nstandard continuous bench set.\n\n## Are there any user-facing changes?\n\nNo.",
          "timestamp": "2026-05-24T17:52:44Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/dae17d6eedbb715a81942cb8616a761ec45eeae3"
        },
        "date": 1779733617816,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_memory_constant_overhead_mib",
            "value": 15.11,
            "unit": "MiB",
            "extra": "Constant memory overhead (C in Memory = C + N*R)"
          },
          {
            "name": "idle_memory_per_core_overhead_mib",
            "value": 0.77,
            "unit": "MiB",
            "extra": "Per-core memory overhead (R in Memory = C + N*R)"
          },
          {
            "name": "idle_memory_r_squared",
            "value": 0.9973,
            "unit": "",
            "extra": "Linear fit quality (R²); 1.0 = perfect linear scaling"
          },
          {
            "name": "idle_memory_1core_mib",
            "value": 16.04,
            "unit": "MiB",
            "extra": "Idle memory at 1 core(s); predicted=15.9 MiB, error=1.0%"
          },
          {
            "name": "idle_memory_2core_mib",
            "value": 16.37,
            "unit": "MiB",
            "extra": "Idle memory at 2 core(s); predicted=16.7 MiB, error=1.8%"
          },
          {
            "name": "idle_memory_4core_mib",
            "value": 17.68,
            "unit": "MiB",
            "extra": "Idle memory at 4 core(s); predicted=18.2 MiB, error=2.9%"
          },
          {
            "name": "idle_memory_8core_mib",
            "value": 21.56,
            "unit": "MiB",
            "extra": "Idle memory at 8 core(s); predicted=21.3 MiB, error=1.3%"
          },
          {
            "name": "idle_memory_16core_mib",
            "value": 28.21,
            "unit": "MiB",
            "extra": "Idle memory at 16 core(s); predicted=27.5 MiB, error=2.6%"
          },
          {
            "name": "idle_memory_32core_mib",
            "value": 39.46,
            "unit": "MiB",
            "extra": "Idle memory at 32 core(s); predicted=39.8 MiB, error=0.9%"
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
          "id": "b163b4888f8faef824a428dae32f4c82c041048b",
          "message": "[Attributes Processor] Add attribute update action (#3084)\n\n# Change Summary\n\nAdds support for the attributes processor `update` action.\n\n`update` replaces existing attribute values for matching keys without\ninserting missing attributes. This provides the generic primitive needed\nfor redaction-style replacements while preserving existing-only\nsemantics.\n\nThe implementation reuses the existing attribute mutation path and\nhandles transport-optimized attribute batches by materializing\n`parent_id` before value changes that can alter equality runs.\n\n\n## What issue does this PR close?\n\n* Closes #3054 \n\n## How are these changes tested?\n\n - `cargo +1.95 fmt --all`\n  - `cargo +1.95 check -p otap-df-pdata`\n\n## Are there any user-facing changes?\n\nYes. Users can configure a new attributes processor `update` action to\nreplace existing attribute values for matching keys without inserting\nmissing attributes.\n\n```yaml\nprocessors:\n  attributes/update:\n    actions:\n      - action: update\n         key: secret\n         value: \"[MASKED]\"\n```\n\n---------\n\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-05-25T20:20:10Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/b163b4888f8faef824a428dae32f4c82c041048b"
        },
        "date": 1779761831813,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_memory_constant_overhead_mib",
            "value": 15.12,
            "unit": "MiB",
            "extra": "Constant memory overhead (C in Memory = C + N*R)"
          },
          {
            "name": "idle_memory_per_core_overhead_mib",
            "value": 0.8,
            "unit": "MiB",
            "extra": "Per-core memory overhead (R in Memory = C + N*R)"
          },
          {
            "name": "idle_memory_r_squared",
            "value": 0.9971,
            "unit": "",
            "extra": "Linear fit quality (R²); 1.0 = perfect linear scaling"
          },
          {
            "name": "idle_memory_1core_mib",
            "value": 15.45,
            "unit": "MiB",
            "extra": "Idle memory at 1 core(s); predicted=15.9 MiB, error=3.0%"
          },
          {
            "name": "idle_memory_2core_mib",
            "value": 17.31,
            "unit": "MiB",
            "extra": "Idle memory at 2 core(s); predicted=16.7 MiB, error=3.4%"
          },
          {
            "name": "idle_memory_4core_mib",
            "value": 18.79,
            "unit": "MiB",
            "extra": "Idle memory at 4 core(s); predicted=18.3 MiB, error=2.5%"
          },
          {
            "name": "idle_memory_8core_mib",
            "value": 20.78,
            "unit": "MiB",
            "extra": "Idle memory at 8 core(s); predicted=21.5 MiB, error=3.5%"
          },
          {
            "name": "idle_memory_16core_mib",
            "value": 27.9,
            "unit": "MiB",
            "extra": "Idle memory at 16 core(s); predicted=27.9 MiB, error=0.1%"
          },
          {
            "name": "idle_memory_32core_mib",
            "value": 40.74,
            "unit": "MiB",
            "extra": "Idle memory at 32 core(s); predicted=40.6 MiB, error=0.2%"
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
          "id": "99fda2c577da533a1840085d37ef4d0445c43d3f",
          "message": "Update dependency kubernetes to v36 (#3082)\n\nThis PR contains the following updates:\n\n| Package | Change |\n[Age](https://docs.renovatebot.com/merge-confidence/) |\n[Confidence](https://docs.renovatebot.com/merge-confidence/) |\n|---|---|---|---|\n| [kubernetes](https://redirect.github.com/kubernetes-client/python) |\n`==35.0.0` → `==36.0.0` |\n![age](https://developer.mend.io/api/mc/badges/age/pypi/kubernetes/36.0.0?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/pypi/kubernetes/35.0.0/36.0.0?slim=true)\n|\n\n---\n\n### Release Notes\n\n<details>\n<summary>kubernetes-client/python (kubernetes)</summary>\n\n###\n[`v36.0.0`](https://redirect.github.com/kubernetes-client/python/blob/HEAD/CHANGELOG.md#v3600)\n\n[Compare\nSource](https://redirect.github.com/kubernetes-client/python/compare/v35.0.0...v36.0.0)\n\nKubernetes API Version: v1.36.1\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: (UTC)\n\n- Branch creation\n  - \"before 8am on Monday\"\n- Automerge\n  - At any time (no schedule defined)\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My4xOTQuMCIsInVwZGF0ZWRJblZlciI6IjQzLjE5NC4wIiwidGFyZ2V0QnJhbmNoIjoibWFpbiIsImxhYmVscyI6WyJkZXBlbmRlbmNpZXMiXX0=-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2026-05-26T16:33:01Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/99fda2c577da533a1840085d37ef4d0445c43d3f"
        },
        "date": 1779819760628,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_memory_constant_overhead_mib",
            "value": 15.26,
            "unit": "MiB",
            "extra": "Constant memory overhead (C in Memory = C + N*R)"
          },
          {
            "name": "idle_memory_per_core_overhead_mib",
            "value": 0.79,
            "unit": "MiB",
            "extra": "Per-core memory overhead (R in Memory = C + N*R)"
          },
          {
            "name": "idle_memory_r_squared",
            "value": 0.9997,
            "unit": "",
            "extra": "Linear fit quality (R²); 1.0 = perfect linear scaling"
          },
          {
            "name": "idle_memory_1core_mib",
            "value": 16.12,
            "unit": "MiB",
            "extra": "Idle memory at 1 core(s); predicted=16.0 MiB, error=0.4%"
          },
          {
            "name": "idle_memory_2core_mib",
            "value": 16.97,
            "unit": "MiB",
            "extra": "Idle memory at 2 core(s); predicted=16.8 MiB, error=0.8%"
          },
          {
            "name": "idle_memory_4core_mib",
            "value": 18.4,
            "unit": "MiB",
            "extra": "Idle memory at 4 core(s); predicted=18.4 MiB, error=0.0%"
          },
          {
            "name": "idle_memory_8core_mib",
            "value": 21.22,
            "unit": "MiB",
            "extra": "Idle memory at 8 core(s); predicted=21.5 MiB, error=1.5%"
          },
          {
            "name": "idle_memory_16core_mib",
            "value": 27.91,
            "unit": "MiB",
            "extra": "Idle memory at 16 core(s); predicted=27.8 MiB, error=0.3%"
          },
          {
            "name": "idle_memory_32core_mib",
            "value": 40.42,
            "unit": "MiB",
            "extra": "Idle memory at 32 core(s); predicted=40.4 MiB, error=0.1%"
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
          "id": "0012b9b79d793aa9412b20f86f79094f99a9590d",
          "message": "fix(security): Revert tidy workflow changes and try different approach (#3098)\n\n# Change Summary\n\nReverts #3056\n\nWhile the intention was correct, the change above made CodeQL and\nOpenSSF very unhappy and flagged the `checkout` in `tidy-commit` as a\nDangerous workflow.\n\nInstead, trying to copy what the opentelemetry-collector maintainers did\nin https://github.com/open-telemetry/opentelemetry-collector/pull/15357.\n\nThis is safe because under the `pull_request` trigger ([GitHub\ndocs](https://docs.github.com/en/actions/writing-workflows/choosing-when-your-workflow-runs/events-that-trigger-workflows#pull_request)):\n\n- **Fork PRs cannot access secrets**, and the `GITHUB_TOKEN` is\nread-only — regardless of what `permissions:` the workflow requests. See\n[Workflows in forked\nrepositories](https://docs.github.com/en/actions/writing-workflows/choosing-when-your-workflow-runs/events-that-trigger-workflows#workflows-in-forked-repositories).\n- **Same-repo PRs** get write access, but the job is gated to\n`renovate[bot]` / `dependabot[bot]` actors and explicitly requires\n`github.event.pull_request.head.repo.fork == false`.",
          "timestamp": "2026-05-27T01:05:04Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/0012b9b79d793aa9412b20f86f79094f99a9590d"
        },
        "date": 1779848540762,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_memory_constant_overhead_mib",
            "value": 15.39,
            "unit": "MiB",
            "extra": "Constant memory overhead (C in Memory = C + N*R)"
          },
          {
            "name": "idle_memory_per_core_overhead_mib",
            "value": 0.77,
            "unit": "MiB",
            "extra": "Per-core memory overhead (R in Memory = C + N*R)"
          },
          {
            "name": "idle_memory_r_squared",
            "value": 0.9972,
            "unit": "",
            "extra": "Linear fit quality (R²); 1.0 = perfect linear scaling"
          },
          {
            "name": "idle_memory_1core_mib",
            "value": 16.32,
            "unit": "MiB",
            "extra": "Idle memory at 1 core(s); predicted=16.2 MiB, error=1.0%"
          },
          {
            "name": "idle_memory_2core_mib",
            "value": 16.73,
            "unit": "MiB",
            "extra": "Idle memory at 2 core(s); predicted=16.9 MiB, error=1.2%"
          },
          {
            "name": "idle_memory_4core_mib",
            "value": 17.9,
            "unit": "MiB",
            "extra": "Idle memory at 4 core(s); predicted=18.5 MiB, error=3.2%"
          },
          {
            "name": "idle_memory_8core_mib",
            "value": 22.4,
            "unit": "MiB",
            "extra": "Idle memory at 8 core(s); predicted=21.6 MiB, error=3.8%"
          },
          {
            "name": "idle_memory_16core_mib",
            "value": 27.51,
            "unit": "MiB",
            "extra": "Idle memory at 16 core(s); predicted=27.7 MiB, error=0.8%"
          },
          {
            "name": "idle_memory_32core_mib",
            "value": 40.01,
            "unit": "MiB",
            "extra": "Idle memory at 32 core(s); predicted=40.0 MiB, error=0.1%"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Cijo Thomas",
            "username": "cijothomas",
            "email": "cijo.thomas@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "f80c48c28097c749fdc0f0b40a58dce4e2bf1034",
          "message": "docs: improve logging macro guidance and event naming references (#3086)\n\nImprove the otel_* logging macro documentation to be accurate and\nconsistent across docs.\n\n- **telemetry README**: Document that event names must follow OTel Event\nnaming conventions (with link to events guide). Note that target maps to\nInstrumentationScope.name and is auto-set to crate name. Replace the\noutdated example with a real call from the codebase.\n- **events-guide**: Add that target becomes InstrumentationScope.name in\nOTLP export. Link the event naming section anchor to the semantic\nconventions guide.",
          "timestamp": "2026-05-27T16:38:49Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/f80c48c28097c749fdc0f0b40a58dce4e2bf1034"
        },
        "date": 1779906868794,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_memory_constant_overhead_mib",
            "value": 14.93,
            "unit": "MiB",
            "extra": "Constant memory overhead (C in Memory = C + N*R)"
          },
          {
            "name": "idle_memory_per_core_overhead_mib",
            "value": 0.82,
            "unit": "MiB",
            "extra": "Per-core memory overhead (R in Memory = C + N*R)"
          },
          {
            "name": "idle_memory_r_squared",
            "value": 0.999,
            "unit": "",
            "extra": "Linear fit quality (R²); 1.0 = perfect linear scaling"
          },
          {
            "name": "idle_memory_1core_mib",
            "value": 15.72,
            "unit": "MiB",
            "extra": "Idle memory at 1 core(s); predicted=15.7 MiB, error=0.2%"
          },
          {
            "name": "idle_memory_2core_mib",
            "value": 16.68,
            "unit": "MiB",
            "extra": "Idle memory at 2 core(s); predicted=16.6 MiB, error=0.7%"
          },
          {
            "name": "idle_memory_4core_mib",
            "value": 18.43,
            "unit": "MiB",
            "extra": "Idle memory at 4 core(s); predicted=18.2 MiB, error=1.2%"
          },
          {
            "name": "idle_memory_8core_mib",
            "value": 21.56,
            "unit": "MiB",
            "extra": "Idle memory at 8 core(s); predicted=21.5 MiB, error=0.2%"
          },
          {
            "name": "idle_memory_16core_mib",
            "value": 27.52,
            "unit": "MiB",
            "extra": "Idle memory at 16 core(s); predicted=28.1 MiB, error=2.2%"
          },
          {
            "name": "idle_memory_32core_mib",
            "value": 41.54,
            "unit": "MiB",
            "extra": "Idle memory at 32 core(s); predicted=41.3 MiB, error=0.6%"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Jake Dern",
            "username": "JakeDern",
            "email": "33842784+JakeDern@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "7e9b8f342c6717d08f85d0fc9ab3275f595bdac1",
          "message": "fix(comparison_dashboard): Fix landing page backpressure detection for a comparison (#3116)\n\n# Change Summary\n\nPull out backpressure detection for a comparison to a helper - There was\nalready a helper for backpressure detection for a test, but not for an\nentire comparison which determines when the warning sign is displayed in\nthe legend.\n\n## What issue does this PR close?\n\n* Closes #3109\n\n## How are these changes tested?\n\n<img width=\"2333\" height=\"712\" alt=\"image\"\nsrc=\"https://github.com/user-attachments/assets/03a1335d-3d13-4275-b08a-0f299ee703d5\"\n/>\n\n## Are there any user-facing changes?\n\nNo",
          "timestamp": "2026-05-27T21:18:42Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/7e9b8f342c6717d08f85d0fc9ab3275f595bdac1"
        },
        "date": 1779936397629,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_memory_constant_overhead_mib",
            "value": 15.06,
            "unit": "MiB",
            "extra": "Constant memory overhead (C in Memory = C + N*R)"
          },
          {
            "name": "idle_memory_per_core_overhead_mib",
            "value": 0.77,
            "unit": "MiB",
            "extra": "Per-core memory overhead (R in Memory = C + N*R)"
          },
          {
            "name": "idle_memory_r_squared",
            "value": 0.9971,
            "unit": "",
            "extra": "Linear fit quality (R²); 1.0 = perfect linear scaling"
          },
          {
            "name": "idle_memory_1core_mib",
            "value": 16.04,
            "unit": "MiB",
            "extra": "Idle memory at 1 core(s); predicted=15.8 MiB, error=1.3%"
          },
          {
            "name": "idle_memory_2core_mib",
            "value": 15.74,
            "unit": "MiB",
            "extra": "Idle memory at 2 core(s); predicted=16.6 MiB, error=5.5%"
          },
          {
            "name": "idle_memory_4core_mib",
            "value": 18.23,
            "unit": "MiB",
            "extra": "Idle memory at 4 core(s); predicted=18.1 MiB, error=0.5%"
          },
          {
            "name": "idle_memory_8core_mib",
            "value": 21.62,
            "unit": "MiB",
            "extra": "Idle memory at 8 core(s); predicted=21.2 MiB, error=1.9%"
          },
          {
            "name": "idle_memory_16core_mib",
            "value": 27.81,
            "unit": "MiB",
            "extra": "Idle memory at 16 core(s); predicted=27.4 MiB, error=1.6%"
          },
          {
            "name": "idle_memory_32core_mib",
            "value": 39.4,
            "unit": "MiB",
            "extra": "Idle memory at 32 core(s); predicted=39.7 MiB, error=0.7%"
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
          "id": "70292bb27def0ed7a179f4cf46ff305dcebd096a",
          "message": "Update one_collect digest to 293b7d3 (#3114)\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n|---|---|---|---|\n| one_collect | workspace.dependencies | digest | `6ccba44` → `293b7d3`\n|\n\n---\n\n### Configuration\n\n📅 **Schedule**: (UTC)\n\n- Branch creation\n  - At any time (no schedule defined)\n- Automerge\n  - At any time (no schedule defined)\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My4xOTQuMCIsInVwZGF0ZWRJblZlciI6IjQzLjE5OC4wIiwidGFyZ2V0QnJhbmNoIjoibWFpbiIsImxhYmVscyI6WyJkZXBlbmRlbmNpZXMiXX0=-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-05-28T17:05:14Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/70292bb27def0ed7a179f4cf46ff305dcebd096a"
        },
        "date": 1779992841245,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_memory_constant_overhead_mib",
            "value": 15.05,
            "unit": "MiB",
            "extra": "Constant memory overhead (C in Memory = C + N*R)"
          },
          {
            "name": "idle_memory_per_core_overhead_mib",
            "value": 0.77,
            "unit": "MiB",
            "extra": "Per-core memory overhead (R in Memory = C + N*R)"
          },
          {
            "name": "idle_memory_r_squared",
            "value": 0.9983,
            "unit": "",
            "extra": "Linear fit quality (R²); 1.0 = perfect linear scaling"
          },
          {
            "name": "idle_memory_1core_mib",
            "value": 15.3,
            "unit": "MiB",
            "extra": "Idle memory at 1 core(s); predicted=15.8 MiB, error=3.4%"
          },
          {
            "name": "idle_memory_2core_mib",
            "value": 17.06,
            "unit": "MiB",
            "extra": "Idle memory at 2 core(s); predicted=16.6 MiB, error=2.8%"
          },
          {
            "name": "idle_memory_4core_mib",
            "value": 17.86,
            "unit": "MiB",
            "extra": "Idle memory at 4 core(s); predicted=18.1 MiB, error=1.5%"
          },
          {
            "name": "idle_memory_8core_mib",
            "value": 21.4,
            "unit": "MiB",
            "extra": "Idle memory at 8 core(s); predicted=21.2 MiB, error=1.0%"
          },
          {
            "name": "idle_memory_16core_mib",
            "value": 27.58,
            "unit": "MiB",
            "extra": "Idle memory at 16 core(s); predicted=27.3 MiB, error=0.9%"
          },
          {
            "name": "idle_memory_32core_mib",
            "value": 39.48,
            "unit": "MiB",
            "extra": "Idle memory at 32 core(s); predicted=39.6 MiB, error=0.4%"
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
          "id": "7d0af33f05ddaf772194302fa68db3f5b9100c64",
          "message": "chore(deps): update one_collect digest to f655a30 (#3130)\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n|---|---|---|---|\n| one_collect | workspace.dependencies | digest | `293b7d3` → `f655a30`\n|\n\n---\n\n### Configuration\n\n📅 **Schedule**: (UTC)\n\n- Branch creation\n  - At any time (no schedule defined)\n- Automerge\n  - At any time (no schedule defined)\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My4xOTguMCIsInVwZGF0ZWRJblZlciI6IjQzLjE5OC4wIiwidGFyZ2V0QnJhbmNoIjoibWFpbiIsImxhYmVscyI6WyJkZXBlbmRlbmNpZXMiXX0=-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2026-05-28T23:19:33Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/7d0af33f05ddaf772194302fa68db3f5b9100c64"
        },
        "date": 1780021404240,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_memory_constant_overhead_mib",
            "value": 15.12,
            "unit": "MiB",
            "extra": "Constant memory overhead (C in Memory = C + N*R)"
          },
          {
            "name": "idle_memory_per_core_overhead_mib",
            "value": 0.8,
            "unit": "MiB",
            "extra": "Per-core memory overhead (R in Memory = C + N*R)"
          },
          {
            "name": "idle_memory_r_squared",
            "value": 0.998,
            "unit": "",
            "extra": "Linear fit quality (R²); 1.0 = perfect linear scaling"
          },
          {
            "name": "idle_memory_1core_mib",
            "value": 15.97,
            "unit": "MiB",
            "extra": "Idle memory at 1 core(s); predicted=15.9 MiB, error=0.3%"
          },
          {
            "name": "idle_memory_2core_mib",
            "value": 16.37,
            "unit": "MiB",
            "extra": "Idle memory at 2 core(s); predicted=16.7 MiB, error=2.1%"
          },
          {
            "name": "idle_memory_4core_mib",
            "value": 18.19,
            "unit": "MiB",
            "extra": "Idle memory at 4 core(s); predicted=18.3 MiB, error=0.7%"
          },
          {
            "name": "idle_memory_8core_mib",
            "value": 21.49,
            "unit": "MiB",
            "extra": "Idle memory at 8 core(s); predicted=21.5 MiB, error=0.1%"
          },
          {
            "name": "idle_memory_16core_mib",
            "value": 28.71,
            "unit": "MiB",
            "extra": "Idle memory at 16 core(s); predicted=27.9 MiB, error=2.8%"
          },
          {
            "name": "idle_memory_32core_mib",
            "value": 40.34,
            "unit": "MiB",
            "extra": "Idle memory at 32 core(s); predicted=40.7 MiB, error=0.9%"
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
          "id": "d624d457aedfdfa11eb8b3d7db98fbc57576a6a8",
          "message": "  chore: document component naming conventions in AGENTS.md (#3125)\n\n# Change Summary\n\nDocument naming conventions for new OTAP Dataflow components in\n`AGENTS.md`, covering consistent module names, component URNs, and\ntelemetry metric set names.\n\n  ## What issue does this PR close?\n\n  None.\n\n  ## How are these changes tested?\n\n  - `npx markdownlint-cli rust/otap-dataflow/AGENTS.md`\n  - `python3 tools/sanitycheck.py`\n  - `git diff --check`\n\n  ## Are there any user-facing changes?\n\n  No. This is contributor/agent guidance only.\n\n  ### Changelog\n\n* [x] Added a `.chloggen/*.yaml` entry, OR this PR is a `chore`\n(indicated in title).\n\n---------\n\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-05-29T15:25:20Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/d624d457aedfdfa11eb8b3d7db98fbc57576a6a8"
        },
        "date": 1780079741362,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_memory_constant_overhead_mib",
            "value": 15.29,
            "unit": "MiB",
            "extra": "Constant memory overhead (C in Memory = C + N*R)"
          },
          {
            "name": "idle_memory_per_core_overhead_mib",
            "value": 0.78,
            "unit": "MiB",
            "extra": "Per-core memory overhead (R in Memory = C + N*R)"
          },
          {
            "name": "idle_memory_r_squared",
            "value": 0.9973,
            "unit": "",
            "extra": "Linear fit quality (R²); 1.0 = perfect linear scaling"
          },
          {
            "name": "idle_memory_1core_mib",
            "value": 16.52,
            "unit": "MiB",
            "extra": "Idle memory at 1 core(s); predicted=16.1 MiB, error=2.7%"
          },
          {
            "name": "idle_memory_2core_mib",
            "value": 16.24,
            "unit": "MiB",
            "extra": "Idle memory at 2 core(s); predicted=16.8 MiB, error=3.7%"
          },
          {
            "name": "idle_memory_4core_mib",
            "value": 18.05,
            "unit": "MiB",
            "extra": "Idle memory at 4 core(s); predicted=18.4 MiB, error=2.0%"
          },
          {
            "name": "idle_memory_8core_mib",
            "value": 21.71,
            "unit": "MiB",
            "extra": "Idle memory at 8 core(s); predicted=21.5 MiB, error=0.9%"
          },
          {
            "name": "idle_memory_16core_mib",
            "value": 28.33,
            "unit": "MiB",
            "extra": "Idle memory at 16 core(s); predicted=27.7 MiB, error=2.1%"
          },
          {
            "name": "idle_memory_32core_mib",
            "value": 39.93,
            "unit": "MiB",
            "extra": "Idle memory at 32 core(s); predicted=40.2 MiB, error=0.7%"
          }
        ]
      }
    ]
  }
}