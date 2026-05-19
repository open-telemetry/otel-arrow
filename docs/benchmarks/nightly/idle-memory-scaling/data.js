window.BENCHMARK_DATA = {
  "lastUpdate": 1779216284083,
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
      }
    ]
  }
}