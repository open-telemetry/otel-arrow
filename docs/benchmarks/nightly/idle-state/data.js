window.BENCHMARK_DATA = {
  "lastUpdate": 1779474344452,
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
        "date": 1779157120125,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.14689917544350198,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.3454237024356081,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 18.241071428571427,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 18.34375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.002115,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle Test Duration"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.7105936585896826,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 1.4307172338602914,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 40.47265625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 41.2109375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.002207,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle Test Duration"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.07859524856599136,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.29283912163214454,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 17.287388392857142,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 17.39453125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.002178,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle Test Duration"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.09695696744583114,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.2476454552535244,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 14.989397321428571,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 15.16796875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.002123,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Test Duration"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.25790844121114315,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.510226512026154,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 21.98046875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 22.36328125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.002217,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle Test Duration"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.35471506142992704,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.7940224159402243,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 29.3515625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 29.5859375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.002246,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle Test Duration"
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
        "date": 1779216280868,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.1475990584233012,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.34986068954782473,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 18.34375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 18.55078125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.002131,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 39.526041666666664,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 39.7578125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.6321796031523161,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 1.4510585305105854,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 40.1953125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 40.79296875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.002187,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 56.009114583333336,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 57.5546875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.11718074094101115,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.2929700272479564,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 15.925223214285714,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 16.09375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.002089,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 38.401041666666664,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 38.53125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.09699599048013782,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.25442602537162423,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 16.498883928571427,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 16.6015625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.002191,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 37.42578125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 37.55078125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.2762573611088695,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.5022557795594301,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 21.455915178571427,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 21.67578125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.002133,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 41.5859375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 42.01171875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.3588845462847839,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.833465192337642,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 28.3125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 28.5703125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.002234,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 46.486979166666664,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 47.34375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle Process RSS (MiB) (Max)"
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
        "date": 1779245075257,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.1863481678699233,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.364049490312038,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 18.4921875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 18.6640625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.002147,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 39.325520833333336,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 39.5703125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.7521229542850627,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 1.4137358020849542,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 41.76171875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 42.23046875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.002256,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 55.580729166666664,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 57.16796875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.11654137218941896,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.28889719626168225,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 16.992745535714285,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 17.20703125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.002133,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 38.455729166666664,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 38.609375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.09743778238298519,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.2614014325755217,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 15.763950892857142,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 15.87109375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.002247,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 37.71875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 37.8203125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.2737688391373274,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.4992886605961554,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 21.375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 21.7734375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.002125,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 41.940104166666664,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 42.36328125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.3768170762252408,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.7761352130228211,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 29.127232142857142,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 29.70703125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.002153,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 46.842447916666664,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 47.796875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle Process RSS (MiB) (Max)"
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
        "date": 1779303395239,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.1743775850327261,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.3648886813015725,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 18.908482142857142,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 19.1875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.002208,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 39.56640625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 39.86328125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.631840148623046,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 1.4920208706487035,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 40.074776785714285,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 40.4921875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.00217,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 55.252604166666664,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 56.4765625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.1154724853070888,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.28358088120815816,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 17.0234375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 17.1640625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.002168,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 38.520833333333336,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 38.70703125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.09649690646076169,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.24834630956088447,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 16.377232142857142,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 16.59375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.002167,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 37.514322916666664,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 37.62109375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.2815660902247447,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.5253449347420758,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 22.045758928571427,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 22.390625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.002187,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 41.7734375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 42.19921875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.42331686108248984,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.7706508875739645,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 29.020647321428573,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 29.45703125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.002189,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 46.498697916666664,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 47.29296875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle Process RSS (MiB) (Max)"
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
        "date": 1779331414885,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.16551825568054218,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.41863013698630136,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 17.099888392857142,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 17.41015625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.002118,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 39.723958333333336,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 40,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.6949680338941416,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 1.4386167146974063,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 40.770089285714285,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 41.3125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.00215,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 56.114583333333336,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 57.765625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.11253303909946746,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.2728554751342517,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 16.146763392857142,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 16.4296875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.002046,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 38.408854166666664,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 38.58203125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.10283237166342335,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.26926920382909175,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 15.574776785714286,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 15.67578125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.002147,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 37.78515625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 37.90234375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.2749617354552237,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.4998645598194131,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 21.213169642857142,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 21.44140625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.002164,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 42.036458333333336,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 42.46875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.3487054209996342,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.8094908921064924,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 28.084263392857142,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 28.69921875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.002128,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 47.19140625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 48.14453125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle Process RSS (MiB) (Max)"
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
        "date": 1779474343799,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.17852012239073062,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.34327363339043765,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 18.412946428571427,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 18.50390625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001823,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 39.545572916666664,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 39.77734375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.711140953225439,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 1.4936488906189178,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 38.602120535714285,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 39.30859375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001832,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 55.657552083333336,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 57.26171875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.13892234449558422,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.3012351063001324,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 16.414620535714285,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 16.484375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001837,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 38.471354166666664,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 38.625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.09775112279432278,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.25727541647205354,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 16.348772321428573,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 16.45703125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001839,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 37.8046875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 37.9296875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.28043993363014075,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.5108636399034343,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 21.43359375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 21.8828125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001819,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 41.971354166666664,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 42.41796875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.383174515395048,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.8124610591900311,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 29.239955357142858,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 29.48828125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001766,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 46.3359375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 47.1875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle Process RSS (MiB) (Max)"
          }
        ]
      }
    ]
  }
}