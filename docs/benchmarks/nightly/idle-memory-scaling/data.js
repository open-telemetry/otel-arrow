window.BENCHMARK_DATA = {
  "lastUpdate": 1780628319595,
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
          "id": "ddeb7d3c651e6fdb1abda29a396062b41b32389c",
          "message": "feat(host metrics receiver) - Add opt-in host load average metrics (#3141)\n\n# Change Summary\n\nAdds opt-in Linux load average metrics to the Rust `host_metrics`\nreceiver.\n\nWhen `families.load.enabled: true`, the receiver reads `/proc/loadavg`\nthrough the existing `host_view.root_path` abstraction and emits the\nCollector-compatible gauges:\n\n  - `system.cpu.load_average.1m`\n  - `system.cpu.load_average.5m`\n  - `system.cpu.load_average.15m`\n\nThe load family defaults to disabled because these metric names are\ndevelopment/experimental and are not registered in Semantic Conventions\n1.41.0.\n\n  ## What issue does this PR close?\n\n  * Closes #3067\n\n  ## How are these changes tested?\n\n- `cargo check -p otap-df-core-nodes --features otap-df-otap/crypto-ring\n--all-targets`\n- `cargo test -p otap-df-core-nodes --features otap-df-otap/crypto-ring\nhost_metrics_receiver` on Linux test VM\n  - `cargo xtask check` on Linux test VM\n- `npx markdownlint-cli\nrust/otap-dataflow/crates/core-nodes/src/receivers/host_metrics_receiver/README.md`\n  - `python3 tools/sanitycheck.py`\n\n  ## Are there any user-facing changes?\n\n  Yes. Users can now enable Linux load average metrics with:\n\n  ```yaml\n  families:\n    load:\n      enabled: true\n      interval: 30s\n```\n\n  ### Changelog\n\n  - [x] Added a .chloggen/*.yaml entry, OR this PR is a chore (indicated in title).",
          "timestamp": "2026-05-29T23:21:41Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/ddeb7d3c651e6fdb1abda29a396062b41b32389c"
        },
        "date": 1780109731893,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_memory_constant_overhead_mib",
            "value": 14.84,
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
            "value": 0.9974,
            "unit": "",
            "extra": "Linear fit quality (R²); 1.0 = perfect linear scaling"
          },
          {
            "name": "idle_memory_1core_mib",
            "value": 16.02,
            "unit": "MiB",
            "extra": "Idle memory at 1 core(s); predicted=15.7 MiB, error=2.3%"
          },
          {
            "name": "idle_memory_2core_mib",
            "value": 15.86,
            "unit": "MiB",
            "extra": "Idle memory at 2 core(s); predicted=16.5 MiB, error=3.8%"
          },
          {
            "name": "idle_memory_4core_mib",
            "value": 17.83,
            "unit": "MiB",
            "extra": "Idle memory at 4 core(s); predicted=18.1 MiB, error=1.5%"
          },
          {
            "name": "idle_memory_8core_mib",
            "value": 21.46,
            "unit": "MiB",
            "extra": "Idle memory at 8 core(s); predicted=21.3 MiB, error=0.6%"
          },
          {
            "name": "idle_memory_16core_mib",
            "value": 28.56,
            "unit": "MiB",
            "extra": "Idle memory at 16 core(s); predicted=27.8 MiB, error=2.5%"
          },
          {
            "name": "idle_memory_32core_mib",
            "value": 40.52,
            "unit": "MiB",
            "extra": "Idle memory at 32 core(s); predicted=40.8 MiB, error=0.8%"
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
          "id": "ddeb7d3c651e6fdb1abda29a396062b41b32389c",
          "message": "feat(host metrics receiver) - Add opt-in host load average metrics (#3141)\n\n# Change Summary\n\nAdds opt-in Linux load average metrics to the Rust `host_metrics`\nreceiver.\n\nWhen `families.load.enabled: true`, the receiver reads `/proc/loadavg`\nthrough the existing `host_view.root_path` abstraction and emits the\nCollector-compatible gauges:\n\n  - `system.cpu.load_average.1m`\n  - `system.cpu.load_average.5m`\n  - `system.cpu.load_average.15m`\n\nThe load family defaults to disabled because these metric names are\ndevelopment/experimental and are not registered in Semantic Conventions\n1.41.0.\n\n  ## What issue does this PR close?\n\n  * Closes #3067\n\n  ## How are these changes tested?\n\n- `cargo check -p otap-df-core-nodes --features otap-df-otap/crypto-ring\n--all-targets`\n- `cargo test -p otap-df-core-nodes --features otap-df-otap/crypto-ring\nhost_metrics_receiver` on Linux test VM\n  - `cargo xtask check` on Linux test VM\n- `npx markdownlint-cli\nrust/otap-dataflow/crates/core-nodes/src/receivers/host_metrics_receiver/README.md`\n  - `python3 tools/sanitycheck.py`\n\n  ## Are there any user-facing changes?\n\n  Yes. Users can now enable Linux load average metrics with:\n\n  ```yaml\n  families:\n    load:\n      enabled: true\n      interval: 30s\n```\n\n  ### Changelog\n\n  - [x] Added a .chloggen/*.yaml entry, OR this PR is a chore (indicated in title).",
          "timestamp": "2026-05-29T23:21:41Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/ddeb7d3c651e6fdb1abda29a396062b41b32389c"
        },
        "date": 1780163276805,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_memory_constant_overhead_mib",
            "value": 15.22,
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
            "value": 0.9963,
            "unit": "",
            "extra": "Linear fit quality (R²); 1.0 = perfect linear scaling"
          },
          {
            "name": "idle_memory_1core_mib",
            "value": 15.6,
            "unit": "MiB",
            "extra": "Idle memory at 1 core(s); predicted=16.0 MiB, error=2.5%"
          },
          {
            "name": "idle_memory_2core_mib",
            "value": 16.52,
            "unit": "MiB",
            "extra": "Idle memory at 2 core(s); predicted=16.8 MiB, error=1.4%"
          },
          {
            "name": "idle_memory_4core_mib",
            "value": 17.98,
            "unit": "MiB",
            "extra": "Idle memory at 4 core(s); predicted=18.3 MiB, error=1.7%"
          },
          {
            "name": "idle_memory_8core_mib",
            "value": 21.99,
            "unit": "MiB",
            "extra": "Idle memory at 8 core(s); predicted=21.4 MiB, error=2.9%"
          },
          {
            "name": "idle_memory_16core_mib",
            "value": 28.26,
            "unit": "MiB",
            "extra": "Idle memory at 16 core(s); predicted=27.5 MiB, error=2.7%"
          },
          {
            "name": "idle_memory_32core_mib",
            "value": 39.28,
            "unit": "MiB",
            "extra": "Idle memory at 32 core(s); predicted=39.8 MiB, error=1.2%"
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
          "id": "ddeb7d3c651e6fdb1abda29a396062b41b32389c",
          "message": "feat(host metrics receiver) - Add opt-in host load average metrics (#3141)\n\n# Change Summary\n\nAdds opt-in Linux load average metrics to the Rust `host_metrics`\nreceiver.\n\nWhen `families.load.enabled: true`, the receiver reads `/proc/loadavg`\nthrough the existing `host_view.root_path` abstraction and emits the\nCollector-compatible gauges:\n\n  - `system.cpu.load_average.1m`\n  - `system.cpu.load_average.5m`\n  - `system.cpu.load_average.15m`\n\nThe load family defaults to disabled because these metric names are\ndevelopment/experimental and are not registered in Semantic Conventions\n1.41.0.\n\n  ## What issue does this PR close?\n\n  * Closes #3067\n\n  ## How are these changes tested?\n\n- `cargo check -p otap-df-core-nodes --features otap-df-otap/crypto-ring\n--all-targets`\n- `cargo test -p otap-df-core-nodes --features otap-df-otap/crypto-ring\nhost_metrics_receiver` on Linux test VM\n  - `cargo xtask check` on Linux test VM\n- `npx markdownlint-cli\nrust/otap-dataflow/crates/core-nodes/src/receivers/host_metrics_receiver/README.md`\n  - `python3 tools/sanitycheck.py`\n\n  ## Are there any user-facing changes?\n\n  Yes. Users can now enable Linux load average metrics with:\n\n  ```yaml\n  families:\n    load:\n      enabled: true\n      interval: 30s\n```\n\n  ### Changelog\n\n  - [x] Added a .chloggen/*.yaml entry, OR this PR is a chore (indicated in title).",
          "timestamp": "2026-05-29T23:21:41Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/ddeb7d3c651e6fdb1abda29a396062b41b32389c"
        },
        "date": 1780195019321,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_memory_constant_overhead_mib",
            "value": 15.37,
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
            "value": 0.9997,
            "unit": "",
            "extra": "Linear fit quality (R²); 1.0 = perfect linear scaling"
          },
          {
            "name": "idle_memory_1core_mib",
            "value": 16.09,
            "unit": "MiB",
            "extra": "Idle memory at 1 core(s); predicted=16.2 MiB, error=0.5%"
          },
          {
            "name": "idle_memory_2core_mib",
            "value": 17.2,
            "unit": "MiB",
            "extra": "Idle memory at 2 core(s); predicted=17.0 MiB, error=1.3%"
          },
          {
            "name": "idle_memory_4core_mib",
            "value": 18.31,
            "unit": "MiB",
            "extra": "Idle memory at 4 core(s); predicted=18.6 MiB, error=1.4%"
          },
          {
            "name": "idle_memory_8core_mib",
            "value": 21.85,
            "unit": "MiB",
            "extra": "Idle memory at 8 core(s); predicted=21.8 MiB, error=0.4%"
          },
          {
            "name": "idle_memory_16core_mib",
            "value": 28.24,
            "unit": "MiB",
            "extra": "Idle memory at 16 core(s); predicted=28.2 MiB, error=0.2%"
          },
          {
            "name": "idle_memory_32core_mib",
            "value": 40.94,
            "unit": "MiB",
            "extra": "Idle memory at 32 core(s); predicted=41.0 MiB, error=0.1%"
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
          "id": "ddeb7d3c651e6fdb1abda29a396062b41b32389c",
          "message": "feat(host metrics receiver) - Add opt-in host load average metrics (#3141)\n\n# Change Summary\n\nAdds opt-in Linux load average metrics to the Rust `host_metrics`\nreceiver.\n\nWhen `families.load.enabled: true`, the receiver reads `/proc/loadavg`\nthrough the existing `host_view.root_path` abstraction and emits the\nCollector-compatible gauges:\n\n  - `system.cpu.load_average.1m`\n  - `system.cpu.load_average.5m`\n  - `system.cpu.load_average.15m`\n\nThe load family defaults to disabled because these metric names are\ndevelopment/experimental and are not registered in Semantic Conventions\n1.41.0.\n\n  ## What issue does this PR close?\n\n  * Closes #3067\n\n  ## How are these changes tested?\n\n- `cargo check -p otap-df-core-nodes --features otap-df-otap/crypto-ring\n--all-targets`\n- `cargo test -p otap-df-core-nodes --features otap-df-otap/crypto-ring\nhost_metrics_receiver` on Linux test VM\n  - `cargo xtask check` on Linux test VM\n- `npx markdownlint-cli\nrust/otap-dataflow/crates/core-nodes/src/receivers/host_metrics_receiver/README.md`\n  - `python3 tools/sanitycheck.py`\n\n  ## Are there any user-facing changes?\n\n  Yes. Users can now enable Linux load average metrics with:\n\n  ```yaml\n  families:\n    load:\n      enabled: true\n      interval: 30s\n```\n\n  ### Changelog\n\n  - [x] Added a .chloggen/*.yaml entry, OR this PR is a chore (indicated in title).",
          "timestamp": "2026-05-29T23:21:41Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/ddeb7d3c651e6fdb1abda29a396062b41b32389c"
        },
        "date": 1780249957344,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_memory_constant_overhead_mib",
            "value": 15.13,
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
            "value": 0.9961,
            "unit": "",
            "extra": "Linear fit quality (R²); 1.0 = perfect linear scaling"
          },
          {
            "name": "idle_memory_1core_mib",
            "value": 15.65,
            "unit": "MiB",
            "extra": "Idle memory at 1 core(s); predicted=15.9 MiB, error=1.6%"
          },
          {
            "name": "idle_memory_2core_mib",
            "value": 16.65,
            "unit": "MiB",
            "extra": "Idle memory at 2 core(s); predicted=16.7 MiB, error=0.2%"
          },
          {
            "name": "idle_memory_4core_mib",
            "value": 18.76,
            "unit": "MiB",
            "extra": "Idle memory at 4 core(s); predicted=18.2 MiB, error=2.8%"
          },
          {
            "name": "idle_memory_8core_mib",
            "value": 20.57,
            "unit": "MiB",
            "extra": "Idle memory at 8 core(s); predicted=21.4 MiB, error=3.8%"
          },
          {
            "name": "idle_memory_16core_mib",
            "value": 28.4,
            "unit": "MiB",
            "extra": "Idle memory at 16 core(s); predicted=27.6 MiB, error=2.9%"
          },
          {
            "name": "idle_memory_32core_mib",
            "value": 39.77,
            "unit": "MiB",
            "extra": "Idle memory at 32 core(s); predicted=40.0 MiB, error=0.7%"
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
          "id": "74ec7428dc1d6fcf58fe69f19cfbb98eb6002ae6",
          "message": "chore(deps): update docker digest updates (#3147)\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n|---|---|---|---|\n| docker.io/rust | stage | digest | `a9cfb75` → `f49565f` |\n| gcr.io/distroless/cc-debian13 | final | digest | `8f960b7` → `e1fd250`\n|\n| golang | stage | digest | `b54cbf5` → `2d6c802` |\n| python | final | digest | `5b3879b` → `c845af9` |\n\n---\n\n### Configuration\n\n📅 **Schedule**: (UTC)\n\n- Branch creation\n  - \"before 8am on the first day of the month\"\n- Automerge\n  - At any time (no schedule defined)\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n👻 **Immortal**: This PR will be recreated if closed unmerged. Get\n[config\nhelp](https://redirect.github.com/renovatebot/renovate/discussions) if\nthat's undesired.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My4yMDIuMSIsInVwZGF0ZWRJblZlciI6IjQzLjIwMi4xIiwidGFyZ2V0QnJhbmNoIjoibWFpbiIsImxhYmVscyI6WyJkZXBlbmRlbmNpZXMiXX0=-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2026-06-01T01:15:06Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/74ec7428dc1d6fcf58fe69f19cfbb98eb6002ae6"
        },
        "date": 1780281570915,
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
            "value": 0.9979,
            "unit": "",
            "extra": "Linear fit quality (R²); 1.0 = perfect linear scaling"
          },
          {
            "name": "idle_memory_1core_mib",
            "value": 15.93,
            "unit": "MiB",
            "extra": "Idle memory at 1 core(s); predicted=16.0 MiB, error=0.2%"
          },
          {
            "name": "idle_memory_2core_mib",
            "value": 16.39,
            "unit": "MiB",
            "extra": "Idle memory at 2 core(s); predicted=16.7 MiB, error=2.2%"
          },
          {
            "name": "idle_memory_4core_mib",
            "value": 18,
            "unit": "MiB",
            "extra": "Idle memory at 4 core(s); predicted=18.3 MiB, error=1.8%"
          },
          {
            "name": "idle_memory_8core_mib",
            "value": 21.98,
            "unit": "MiB",
            "extra": "Idle memory at 8 core(s); predicted=21.5 MiB, error=2.2%"
          },
          {
            "name": "idle_memory_16core_mib",
            "value": 28.4,
            "unit": "MiB",
            "extra": "Idle memory at 16 core(s); predicted=27.8 MiB, error=2.0%"
          },
          {
            "name": "idle_memory_32core_mib",
            "value": 40.13,
            "unit": "MiB",
            "extra": "Idle memory at 32 core(s); predicted=40.5 MiB, error=0.9%"
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
          "id": "43bb12cc2b86b6bbbdfbfd48431609a186a4b52b",
          "message": "Add OTAP-ATTR-OTAP saturation scaling tests (1,2,4,8 cores) (#3145)\n\n## Summary\n\nAdd OTAP saturation/scaling tests to measure OTAP throughput scaling\nacross core counts, matching the existing OTLP scaling tests.\n\n**Config:** OTAP receiver → attribute rename processor → OTAP exporter,\nzstd compression, batch size 512, 1KB synthetic logs.\n\n**Core layouts (NUMA-aware):**\n| Cores | SUT | Loadgen | Backend |\n|-------|-----|---------|---------|\n| 1 | 0 (1) | 32-41 (10) | 42-43 (2) |\n| 2 | 0-1 (2) | 32-51 (20) | 52-55 (4) |\n| 4 | 0-3 (4) | 32-63,96-119 (56) | 120-127 (8) |\n| 8 | 0-7 (8) | 32-63,96-127 (64) | 64-79 (16) |\n\n## Results (3 runs, highly consistent ±2%)\n\n| Cores | Throughput | SUT CPU | LG CPU (allocated) | BE CPU (allocated)\n| Scaling |\n|-------|-----------|---------|-------|-------|---------|\n| 1 | **2.47M** logs/sec | 100% | 10.0/10 ✅ | 0.63/2 | 100% |\n| 2 | **4.82M** logs/sec | 100% | 20.0/20 ✅ | 0.71/4 | 97% |\n| 4 | **9.04M** logs/sec | 100% ✅ | 56.0/56 ✅ | 2.35/8 | 92% |\n| 8 | **14.1M** logs/sec | 92% ⚠️ | 64.0/64 ⚠️ | 2.87/16 | 72% |\n\n### Analysis\n- **1-4 cores: fully saturated** — near-linear scaling (92-100%)\n- **8 cores: loadgen bottleneck** — used all 64 allocated cores, SUT\nonly at 92%. True throughput is higher.\n- **Backend is never the bottleneck** — peaks at ~3 cores out of 16\nallocated\n- **Per-core throughput: ~2.4-2.5M logs/sec** (vs OTLP ~120K — **~20x\nfaster**)\n\n### 8-core loadgen limitation\n\nThe 8-core SUT cannot be fully saturated due to a NUMA topology\nconstraint:\n- **CI machine:** 2-socket Intel Xeon 8358, 2 NUMA nodes × 64 logical\ncores (32 physical + 32 HT)\n- **SUT on NUMA0** (cores 0-7), **Loadgen on NUMA1** (cores 32-63,\n96-127 = 64 cores)\n- Placing loadgen on the same NUMA node as SUT causes significant\nthroughput reduction (tested: 14M → 8.7M)\n- 64 cores is the maximum loadgen allocation without cross-NUMA\ncontention\n- Each loadgen core produces ~220K logs/sec → 64 × 220K ≈ 14M,\ninsufficient for 8 × 2.5M = 20M theoretical max\n\n### Comparison with OTLP scaling (same test)\n| Cores | OTLP | OTAP | Speedup |\n|-------|------|------|---------|\n| 1 | 121K | 2.47M | 20.4x |\n| 2 | 264K | 4.82M | 18.3x |\n| 4 | 567K | 9.04M | 15.9x |\n| 8 | 1.03M | 14.1M | 13.7x |",
          "timestamp": "2026-06-01T18:19:04Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/43bb12cc2b86b6bbbdfbfd48431609a186a4b52b"
        },
        "date": 1780342144162,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_memory_constant_overhead_mib",
            "value": 14.83,
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
            "value": 0.9985,
            "unit": "",
            "extra": "Linear fit quality (R²); 1.0 = perfect linear scaling"
          },
          {
            "name": "idle_memory_1core_mib",
            "value": 15.71,
            "unit": "MiB",
            "extra": "Idle memory at 1 core(s); predicted=15.6 MiB, error=0.5%"
          },
          {
            "name": "idle_memory_2core_mib",
            "value": 16.64,
            "unit": "MiB",
            "extra": "Idle memory at 2 core(s); predicted=16.4 MiB, error=1.2%"
          },
          {
            "name": "idle_memory_4core_mib",
            "value": 17.7,
            "unit": "MiB",
            "extra": "Idle memory at 4 core(s); predicted=18.1 MiB, error=2.0%"
          },
          {
            "name": "idle_memory_8core_mib",
            "value": 20.97,
            "unit": "MiB",
            "extra": "Idle memory at 8 core(s); predicted=21.3 MiB, error=1.5%"
          },
          {
            "name": "idle_memory_16core_mib",
            "value": 28.36,
            "unit": "MiB",
            "extra": "Idle memory at 16 core(s); predicted=27.8 MiB, error=2.1%"
          },
          {
            "name": "idle_memory_32core_mib",
            "value": 40.48,
            "unit": "MiB",
            "extra": "Idle memory at 32 core(s); predicted=40.7 MiB, error=0.5%"
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
          "id": "b31a3c75011eccdc455a74fcc4a9838eefc5a6da",
          "message": "fix(deps): update module go.opentelemetry.io/collector/pdata to v1.59.0 (#3162)\n\nThis PR contains the following updates:\n\n| Package | Change |\n[Age](https://docs.renovatebot.com/merge-confidence/) |\n[Confidence](https://docs.renovatebot.com/merge-confidence/) |\n|---|---|---|---|\n|\n[go.opentelemetry.io/collector/pdata](https://redirect.github.com/open-telemetry/opentelemetry-collector)\n| `v1.58.0` → `v1.59.0` |\n![age](https://developer.mend.io/api/mc/badges/age/go/go.opentelemetry.io%2fcollector%2fpdata/v1.59.0?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/go/go.opentelemetry.io%2fcollector%2fpdata/v1.58.0/v1.59.0?slim=true)\n|\n\n---\n\n### Release Notes\n\n<details>\n<summary>open-telemetry/opentelemetry-collector\n(go.opentelemetry.io/collector/pdata)</summary>\n\n###\n[`v1.59.0`](https://redirect.github.com/open-telemetry/opentelemetry-collector/blob/HEAD/CHANGELOG.md#v1590v01530)\n\n##### 🛑 Breaking changes 🛑\n\n- `pkg/configoptional`: Stabilize feature gate\nconfigoptional.AddEnabledField\n([#&#8203;15333](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/15333))\n- `pkg/confmap`: Stabilize confmap.newExpandedValueSanitizer feature\ngate\n([#&#8203;15339](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/15339))\n- `pkg/exporterhelper`: mark exporter.PersistRequestContext as stable\n([#&#8203;15330](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/15330))\n- `pkg/otelcol`: Stabilize otelcol.printInitialConfig gate\n([#&#8203;15340](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/15340))\n- `pkg/pdata`: Remove pdata.useCustomProtoEncoding feature gate\n([#&#8203;15332](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/15332))\n- `pkg/service`: Stabilize telemetry.UseLocalHostAsDefaultMetricsAddress\ngate\n([#&#8203;15342](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/15342))\n- `pkg/xpdata`: Stabilize pdata.enableRefCounting feature gate\n([#&#8203;15331](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/15331))\n\n##### 🧰 Bug fixes 🧰\n\n- `pkg/config/configgrpc`: Fix memory corruption and fatal error in\nSnappy\n([#&#8203;15237](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/15237),\n[#&#8203;15320](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/15320))\n\n<!-- previous-version -->\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: (UTC)\n\n- Branch creation\n  - \"before 8am on Monday\"\n- Automerge\n  - At any time (no schedule defined)\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My4yMDYuMSIsInVwZGF0ZWRJblZlciI6IjQzLjIwNi4xIiwidGFyZ2V0QnJhbmNoIjoibWFpbiIsImxhYmVscyI6WyJkZXBlbmRlbmNpZXMiXX0=-->\n\n---------\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: otelbot <197425009+otelbot@users.noreply.github.com>",
          "timestamp": "2026-06-01T23:18:45Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/b31a3c75011eccdc455a74fcc4a9838eefc5a6da"
        },
        "date": 1780370319135,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_memory_constant_overhead_mib",
            "value": 14.91,
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
            "value": 0.999,
            "unit": "",
            "extra": "Linear fit quality (R²); 1.0 = perfect linear scaling"
          },
          {
            "name": "idle_memory_1core_mib",
            "value": 15.76,
            "unit": "MiB",
            "extra": "Idle memory at 1 core(s); predicted=15.7 MiB, error=0.4%"
          },
          {
            "name": "idle_memory_2core_mib",
            "value": 16.34,
            "unit": "MiB",
            "extra": "Idle memory at 2 core(s); predicted=16.5 MiB, error=0.8%"
          },
          {
            "name": "idle_memory_4core_mib",
            "value": 17.83,
            "unit": "MiB",
            "extra": "Idle memory at 4 core(s); predicted=18.0 MiB, error=1.2%"
          },
          {
            "name": "idle_memory_8core_mib",
            "value": 21.13,
            "unit": "MiB",
            "extra": "Idle memory at 8 core(s); predicted=21.2 MiB, error=0.2%"
          },
          {
            "name": "idle_memory_16core_mib",
            "value": 27.95,
            "unit": "MiB",
            "extra": "Idle memory at 16 core(s); predicted=27.4 MiB, error=1.9%"
          },
          {
            "name": "idle_memory_32core_mib",
            "value": 39.7,
            "unit": "MiB",
            "extra": "Idle memory at 32 core(s); predicted=39.9 MiB, error=0.6%"
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
          "id": "39d17768106ad32f00b474b9c75b6b8fd740206b",
          "message": "df_engine: MVP weaver live-check for in-tree semconv events (#1613) (#3047)\n\nInitial wiring for #1613: validates one internal event\n(`tls.handshake.failed`) end-to-end via `weaver registry live-check` in\nCI. Establishes the registry → emit → live-check mechanism so subsequent\nPRs can backfill the remaining events and tighten gates.\n\nBuilds on #3049 (event_name / InstrumentationScope.name encoder change),\nwhich is already merged.\n\n## Changes\n\n- New in-tree semconv registry at `rust/otap-dataflow/semconv/` with one\n`type: event` group. `manifest.yaml` pulls upstream\n`semantic-conventions@v1.41.0` via `dependencies:` (no second checkout\nin CI).\n- New `configs/internal-events-otlp.yaml` wires `internal_telemetry` →\n`otlp_grpc` and a TLS-enabled `otlp` receiver whose handshakes are\nfailed by plaintext HTTP from CI.\n- New workflow `.github/workflows/df-engine-internal-observability.yml`\nholding the live-check job. Kept out of `rust-ci.yml` to convey\nlong-term intent and avoid polluting the rust workflow as the registry\ngrows. **Not in required status checks yet**, mirroring the staged\nrollout used for the host-metrics live-check. The assert step is\nregistry-driven: it discovers every declared event and fails if any\nreceived zero samples or has event-level violations.\n\n## Verified locally\n\n`weaver registry check` clean; `cargo xtask check` green; end-to-end\nsmoke produced `tls.handshake.failed` samples with 0 event-level\nviolations.\n\n## Deferred (follow-ups under #1613)\n\nBackfill remaining event names; attribute-level alignment (e.g. `error`\n→ `error.type` per OTel semconv); `InstrumentationScope.version`; xtask\nstatic drift check; promote workflow to required. Severity declaration\nin semconv is blocked on open-telemetry/weaver#1004 (the wire already\ncarries `severity_number = 13`).\n\nFollow-up: adopt `weaver-live-check-{start,stop}` composite actions once\nopen-telemetry/weaver#1448 merges to drop ~half the workflow\nboilerplate.",
          "timestamp": "2026-06-02T17:58:58Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/39d17768106ad32f00b474b9c75b6b8fd740206b"
        },
        "date": 1780427538359,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_memory_constant_overhead_mib",
            "value": 14.46,
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
            "value": 0.9979,
            "unit": "",
            "extra": "Linear fit quality (R²); 1.0 = perfect linear scaling"
          },
          {
            "name": "idle_memory_1core_mib",
            "value": 15.58,
            "unit": "MiB",
            "extra": "Idle memory at 1 core(s); predicted=15.3 MiB, error=1.8%"
          },
          {
            "name": "idle_memory_2core_mib",
            "value": 16.45,
            "unit": "MiB",
            "extra": "Idle memory at 2 core(s); predicted=16.1 MiB, error=1.9%"
          },
          {
            "name": "idle_memory_4core_mib",
            "value": 17.82,
            "unit": "MiB",
            "extra": "Idle memory at 4 core(s); predicted=17.8 MiB, error=0.1%"
          },
          {
            "name": "idle_memory_8core_mib",
            "value": 20.22,
            "unit": "MiB",
            "extra": "Idle memory at 8 core(s); predicted=21.1 MiB, error=4.5%"
          },
          {
            "name": "idle_memory_16core_mib",
            "value": 27.96,
            "unit": "MiB",
            "extra": "Idle memory at 16 core(s); predicted=27.8 MiB, error=0.6%"
          },
          {
            "name": "idle_memory_32core_mib",
            "value": 41.22,
            "unit": "MiB",
            "extra": "Idle memory at 32 core(s); predicted=41.1 MiB, error=0.3%"
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
          "id": "9fb01d6d441d84fcdb5a0a75d377f041fa0cdfca",
          "message": "Add uncompressed bytes-per-log metric to traffic generator benchmarks (#3026)\n\nCloses #2987\n\nAdds a `logs_bytes_produced` counter metric to the traffic generator\nreceiver that tracks the total protobuf-encoded (uncompressed) bytes of\nlog payloads produced. The benchmark report SQL then computes\n`uncompressed_bytes_per_log` from this counter, enabling direct\ncomparison of uncompressed payload size against the egress (compressed)\nbytes per log.\n\n### Changes\n- **metrics.rs**: Added `logs_bytes_produced: Counter<u64>` with unit\n`By`\n- **mod.rs**: Record payload bytes in `export_pdata()` for log signals\n(captured before ownership move)\n- **integration_report_logs.yaml** & **report_logs.yaml**: Added\n`logs_bytes_produced` to metric filter and `uncompressed_bytes_per_log`\ncomputed metric to report SQL",
          "timestamp": "2026-06-02T22:17:51Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/9fb01d6d441d84fcdb5a0a75d377f041fa0cdfca"
        },
        "date": 1780455980466,
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
            "value": 0.78,
            "unit": "MiB",
            "extra": "Per-core memory overhead (R in Memory = C + N*R)"
          },
          {
            "name": "idle_memory_r_squared",
            "value": 0.9966,
            "unit": "",
            "extra": "Linear fit quality (R²); 1.0 = perfect linear scaling"
          },
          {
            "name": "idle_memory_1core_mib",
            "value": 16.46,
            "unit": "MiB",
            "extra": "Idle memory at 1 core(s); predicted=15.9 MiB, error=3.4%"
          },
          {
            "name": "idle_memory_2core_mib",
            "value": 16.39,
            "unit": "MiB",
            "extra": "Idle memory at 2 core(s); predicted=16.7 MiB, error=1.8%"
          },
          {
            "name": "idle_memory_4core_mib",
            "value": 17.48,
            "unit": "MiB",
            "extra": "Idle memory at 4 core(s); predicted=18.2 MiB, error=4.3%"
          },
          {
            "name": "idle_memory_8core_mib",
            "value": 21.44,
            "unit": "MiB",
            "extra": "Idle memory at 8 core(s); predicted=21.4 MiB, error=0.4%"
          },
          {
            "name": "idle_memory_16core_mib",
            "value": 28.23,
            "unit": "MiB",
            "extra": "Idle memory at 16 core(s); predicted=27.6 MiB, error=2.3%"
          },
          {
            "name": "idle_memory_32core_mib",
            "value": 39.82,
            "unit": "MiB",
            "extra": "Idle memory at 32 core(s); predicted=40.1 MiB, error=0.6%"
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
          "id": "a7a04541634b883904108e499e52bbeb94ccfb6e",
          "message": "feat: added comment support in OPL (#3152)\n\n# Change Summary\n\n<!--\nReplace with a brief summary of the change in this PR\n-->\n\nAdds support for comments in OPL programs. Both inline comments (`//`)\nand block comments (`/* ... */`) are supported\n\n## What issue does this PR close?\n\n<!--\nWe highly recommend correlation of every PR to an issue\n-->\n\n* Closes https://github.com/open-telemetry/otel-arrow/issues/3151\n\n## How are these changes tested?\n\nUnit\n\n## Are there any user-facing changes?\n\nYes - this comment syntax is now available for OPL programs written in\nthe transform processor config.\n\n <!-- If yes, provide further info below -->\n\n### Changelog\n\n<!--\nUser-facing changes need a .chloggen/*.yaml entry. Copy the\nTEMPLATE.yaml\nin go/.chloggen/ or rust/otap-dataflow/.chloggen/ and fill in the\nfields.\nIf not required, include `chore` in the PR title.\n-->\n\n* [x] Added a `.chloggen/*.yaml` entry, OR this PR is a `chore`\n(indicated in title).",
          "timestamp": "2026-06-03T18:14:18Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/a7a04541634b883904108e499e52bbeb94ccfb6e"
        },
        "date": 1780514073488,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_memory_constant_overhead_mib",
            "value": 15.38,
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
            "value": 0.9989,
            "unit": "",
            "extra": "Linear fit quality (R²); 1.0 = perfect linear scaling"
          },
          {
            "name": "idle_memory_1core_mib",
            "value": 16.31,
            "unit": "MiB",
            "extra": "Idle memory at 1 core(s); predicted=16.2 MiB, error=0.9%"
          },
          {
            "name": "idle_memory_2core_mib",
            "value": 16.42,
            "unit": "MiB",
            "extra": "Idle memory at 2 core(s); predicted=16.9 MiB, error=3.1%"
          },
          {
            "name": "idle_memory_4core_mib",
            "value": 18.62,
            "unit": "MiB",
            "extra": "Idle memory at 4 core(s); predicted=18.5 MiB, error=0.7%"
          },
          {
            "name": "idle_memory_8core_mib",
            "value": 21.62,
            "unit": "MiB",
            "extra": "Idle memory at 8 core(s); predicted=21.6 MiB, error=0.2%"
          },
          {
            "name": "idle_memory_16core_mib",
            "value": 28.15,
            "unit": "MiB",
            "extra": "Idle memory at 16 core(s); predicted=27.8 MiB, error=1.3%"
          },
          {
            "name": "idle_memory_32core_mib",
            "value": 39.98,
            "unit": "MiB",
            "extra": "Idle memory at 32 core(s); predicted=40.2 MiB, error=0.5%"
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
          "id": "ee0b8953fa04dee164e3f9149751559be8b05f2e",
          "message": "task(comparison_dashboard): Add comparisons to fluent bit (#3190)\n\n# Change Summary\n\nThis PR adds comparisons to fluent bit for logs across all comparisons.\nMetrics and traces are not included though we may want to add them in\nthe future as I think there is some support there.\n\n## What issue does this PR close?\n\n* Closes #3169\n\n## How are these changes tested?\n\n<img width=\"2335\" height=\"1572\" alt=\"image\"\nsrc=\"https://github.com/user-attachments/assets/17be789e-6aca-40da-aefb-b0fac929da7d\"\n/>\n\n## Are there any user-facing changes?\n\nYes new comparisons on the dashboard.\n\n### Changelog\n\n<!--\nUser-facing changes need a .chloggen/*.yaml entry. Copy the\nTEMPLATE.yaml\nin go/.chloggen/ or rust/otap-dataflow/.chloggen/ and fill in the\nfields.\nIf not required, include `chore` in the PR title.\n-->\n\n* [x] Added a `.chloggen/*.yaml` entry, OR this PR is a `chore`\n(indicated in title).",
          "timestamp": "2026-06-04T01:18:21Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/ee0b8953fa04dee164e3f9149751559be8b05f2e"
        },
        "date": 1780543779934,
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
            "value": 0.76,
            "unit": "MiB",
            "extra": "Per-core memory overhead (R in Memory = C + N*R)"
          },
          {
            "name": "idle_memory_r_squared",
            "value": 0.9978,
            "unit": "",
            "extra": "Linear fit quality (R²); 1.0 = perfect linear scaling"
          },
          {
            "name": "idle_memory_1core_mib",
            "value": 15.61,
            "unit": "MiB",
            "extra": "Idle memory at 1 core(s); predicted=16.0 MiB, error=2.5%"
          },
          {
            "name": "idle_memory_2core_mib",
            "value": 16.61,
            "unit": "MiB",
            "extra": "Idle memory at 2 core(s); predicted=16.8 MiB, error=0.9%"
          },
          {
            "name": "idle_memory_4core_mib",
            "value": 18.45,
            "unit": "MiB",
            "extra": "Idle memory at 4 core(s); predicted=18.3 MiB, error=0.9%"
          },
          {
            "name": "idle_memory_8core_mib",
            "value": 21.33,
            "unit": "MiB",
            "extra": "Idle memory at 8 core(s); predicted=21.3 MiB, error=0.1%"
          },
          {
            "name": "idle_memory_16core_mib",
            "value": 28.13,
            "unit": "MiB",
            "extra": "Idle memory at 16 core(s); predicted=27.4 MiB, error=2.6%"
          },
          {
            "name": "idle_memory_32core_mib",
            "value": 39.18,
            "unit": "MiB",
            "extra": "Idle memory at 32 core(s); predicted=39.5 MiB, error=0.9%"
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
          "id": "c6ed105cab28e537bf5c2c81a97e9b63677d3cff",
          "message": "fix(comparison_dashboard): Remove fluent bit otlpgrpc gzip/zstd suites (#3200)\n\n# Change Summary\n\nFluent bit seems to ignore compression settings on the otlp grpc path,\nso removing those suites.",
          "timestamp": "2026-06-04T03:45:28Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/c6ed105cab28e537bf5c2c81a97e9b63677d3cff"
        },
        "date": 1780600548024,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_memory_constant_overhead_mib",
            "value": 15.51,
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
            "value": 0.9928,
            "unit": "",
            "extra": "Linear fit quality (R²); 1.0 = perfect linear scaling"
          },
          {
            "name": "idle_memory_1core_mib",
            "value": 16.73,
            "unit": "MiB",
            "extra": "Idle memory at 1 core(s); predicted=16.3 MiB, error=2.8%"
          },
          {
            "name": "idle_memory_2core_mib",
            "value": 16.07,
            "unit": "MiB",
            "extra": "Idle memory at 2 core(s); predicted=17.0 MiB, error=6.0%"
          },
          {
            "name": "idle_memory_4core_mib",
            "value": 18.53,
            "unit": "MiB",
            "extra": "Idle memory at 4 core(s); predicted=18.6 MiB, error=0.1%"
          },
          {
            "name": "idle_memory_8core_mib",
            "value": 21.4,
            "unit": "MiB",
            "extra": "Idle memory at 8 core(s); predicted=21.6 MiB, error=0.9%"
          },
          {
            "name": "idle_memory_16core_mib",
            "value": 28.92,
            "unit": "MiB",
            "extra": "Idle memory at 16 core(s); predicted=27.7 MiB, error=4.2%"
          },
          {
            "name": "idle_memory_32core_mib",
            "value": 39.36,
            "unit": "MiB",
            "extra": "Idle memory at 32 core(s); predicted=39.9 MiB, error=1.3%"
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
          "id": "c6ed105cab28e537bf5c2c81a97e9b63677d3cff",
          "message": "fix(comparison_dashboard): Remove fluent bit otlpgrpc gzip/zstd suites (#3200)\n\n# Change Summary\n\nFluent bit seems to ignore compression settings on the otlp grpc path,\nso removing those suites.",
          "timestamp": "2026-06-04T03:45:28Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/c6ed105cab28e537bf5c2c81a97e9b63677d3cff"
        },
        "date": 1780628318882,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_memory_constant_overhead_mib",
            "value": 14.97,
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
            "value": 0.998,
            "unit": "",
            "extra": "Linear fit quality (R²); 1.0 = perfect linear scaling"
          },
          {
            "name": "idle_memory_1core_mib",
            "value": 15.56,
            "unit": "MiB",
            "extra": "Idle memory at 1 core(s); predicted=15.8 MiB, error=1.3%"
          },
          {
            "name": "idle_memory_2core_mib",
            "value": 16.8,
            "unit": "MiB",
            "extra": "Idle memory at 2 core(s); predicted=16.6 MiB, error=1.5%"
          },
          {
            "name": "idle_memory_4core_mib",
            "value": 18.44,
            "unit": "MiB",
            "extra": "Idle memory at 4 core(s); predicted=18.1 MiB, error=1.6%"
          },
          {
            "name": "idle_memory_8core_mib",
            "value": 20.59,
            "unit": "MiB",
            "extra": "Idle memory at 8 core(s); predicted=21.3 MiB, error=3.4%"
          },
          {
            "name": "idle_memory_16core_mib",
            "value": 28.06,
            "unit": "MiB",
            "extra": "Idle memory at 16 core(s); predicted=27.6 MiB, error=1.6%"
          },
          {
            "name": "idle_memory_32core_mib",
            "value": 40.18,
            "unit": "MiB",
            "extra": "Idle memory at 32 core(s); predicted=40.3 MiB, error=0.2%"
          }
        ]
      }
    ]
  }
}