window.BENCHMARK_DATA = {
  "lastUpdate": 1780628316749,
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
        "date": 1779507448440,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.13407324303746926,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.3495162544719241,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 18.629464285714285,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 18.890625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001799,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 39.41015625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 39.66015625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.694780697200329,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 1.4183783783783785,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 41.579799107142854,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 42.16015625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.003263,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 55.881510416666664,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 57.44921875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.11067491516639448,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.26863951405653763,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 16.517857142857142,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 16.625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.00183,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 38.12109375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 38.27734375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.0984438899521369,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.25771477529402603,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 15.76171875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 15.8671875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001808,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 38.041666666666664,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 38.12109375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.26649127883376333,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.5234189631013545,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 21.859933035714285,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 22.1328125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001791,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 41.686197916666664,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 42.1015625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.35034538182141794,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.7984927987543791,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 28.352678571428573,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 28.73046875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001782,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 46.544270833333336,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 47.40234375,
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
        "date": 1779557611325,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.14561411972236563,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.3479853593956857,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 17.744419642857142,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 17.8515625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001786,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 39.776041666666664,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 40.03125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.6589021658973403,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 1.37130042049525,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 40.804129464285715,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 41.484375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001931,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 56.03515625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 57.578125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.11914978985698271,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.30784179383369664,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 16.256138392857142,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 16.36328125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001857,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 38.619791666666664,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 38.7421875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.07227419977528216,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.2630546473610463,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 15.659040178571429,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 15.69921875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001823,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 37.555989583333336,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 37.66015625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.2377215744507137,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.5164440292698116,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 21.442522321428573,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 21.69921875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.00183,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 41.678385416666664,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 42.0859375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.4324543736145814,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.821128316862501,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 27.482142857142858,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 27.6015625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001836,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 46.9609375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 47.68359375,
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
        "date": 1779589168574,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.1849434844214726,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.37657609964966915,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 19.162946428571427,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 19.546875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001785,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 39.739583333333336,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 39.9921875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.7134948609134654,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 1.4779196511990036,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 39.88671875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 40.265625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001814,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 55.709635416666664,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 56.86328125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.13682350934942633,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.3228748451053284,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 17.450334821428573,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 17.69140625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001861,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 37.893229166666664,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 38.04296875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.07683925354376783,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.25873082613096626,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 15.482700892857142,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 15.58984375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001769,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 37.416666666666664,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 37.515625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.2513739746546063,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.5012378357337486,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 22.8125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 23.03125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001947,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 41.399739583333336,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 41.83984375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.3472138426294092,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.7644748716352887,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 28.303013392857142,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 28.63671875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001868,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 47.00390625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 47.9453125,
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
        "date": 1779644043065,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.15168146947873148,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.36282444530945895,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 17.870535714285715,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 18.078125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001827,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 39.927083333333336,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 40.16015625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.703992808341758,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 1.3363885429638855,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 39.951450892857146,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 40.37890625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001866,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 55.64453125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 57.17578125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.11316503139440577,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.28039234002802427,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 16.486607142857142,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 16.59375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.00177,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 38.15625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 38.28515625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.1008239807432992,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.28878474114441416,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 16.320870535714285,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 16.53515625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001745,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 38.125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 38.2109375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.2523940121954956,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.523120037365717,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 21.208147321428573,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 21.390625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001788,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 41.434895833333336,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 41.86328125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.38407240491838435,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.7989415518717411,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 27.351004464285715,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 27.48046875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001755,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 46.807291666666664,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 47.51953125,
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
          "id": "dae17d6eedbb715a81942cb8616a761ec45eeae3",
          "message": "perf: Box inner raw_batch_store record batch slices (#3077)\n\n# Change Summary\n\nBoxes raw_batch_store record batch slices. Attached issue explains the\nrationale, but basically, we have very a very large enum variant for\notap metrics (almost 1k) which is penalizing a lot of data structures.\n\nThis penalty is more visible when queues are larger and/or saturated,\nwhen batches are smaller in size, or when there are many processing\nstages.\n\n## What issue does this PR close?\n\n* Closes #3076\n\n## How are these changes tested?\n\nAd-hoc perf testing and also the manual pipelineperf run for the\nstandard continuous bench set.\n\n## Are there any user-facing changes?\n\nNo.",
          "timestamp": "2026-05-24T17:52:44Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/dae17d6eedbb715a81942cb8616a761ec45eeae3"
        },
        "date": 1779675700196,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.15122511739700709,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.35011442360084066,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 17.880022321428573,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 17.98828125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001863,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 39.372395833333336,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 39.59765625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.5838980401217954,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 1.3972610422996028,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 39.546875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 39.82421875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001894,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 54.184895833333336,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 55.3125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.11587816253417998,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.28413449564134496,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 15.942522321428571,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 15.9453125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001884,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 38.2578125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 38.41015625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.09510113529429307,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.25655795127267067,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 16.58984375,
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
            "value": 15.001954,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 37.458333333333336,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 37.5390625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.2571192552550083,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.47793178632611744,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 21.471540178571427,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 21.71484375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.00181,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 41.720052083333336,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 42.125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.3901166831400166,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.7685382758352153,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 29.001674107142858,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 29.44140625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.0019,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 46.157552083333336,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 47.05859375,
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
          "id": "dae17d6eedbb715a81942cb8616a761ec45eeae3",
          "message": "perf: Box inner raw_batch_store record batch slices (#3077)\n\n# Change Summary\n\nBoxes raw_batch_store record batch slices. Attached issue explains the\nrationale, but basically, we have very a very large enum variant for\notap metrics (almost 1k) which is penalizing a lot of data structures.\n\nThis penalty is more visible when queues are larger and/or saturated,\nwhen batches are smaller in size, or when there are many processing\nstages.\n\n## What issue does this PR close?\n\n* Closes #3076\n\n## How are these changes tested?\n\nAd-hoc perf testing and also the manual pipelineperf run for the\nstandard continuous bench set.\n\n## Are there any user-facing changes?\n\nNo.",
          "timestamp": "2026-05-24T17:52:44Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/dae17d6eedbb715a81942cb8616a761ec45eeae3"
        },
        "date": 1779733615502,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.14524222022466435,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.34185159230709333,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 17.684709821428573,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 17.87109375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001798,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 39.33984375,
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
            "value": 0.6772724995746854,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 1.3568876439464674,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 39.462611607142854,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 40.0234375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001976,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 54.279947916666664,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 55.80859375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.11350393870237435,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.28485405152953996,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 16.366071428571427,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 16.3671875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001797,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 38.296875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 38.42578125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.1139862233898037,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.27184960770605143,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 16.040178571428573,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 16.1484375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.002143,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 37.505208333333336,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 37.609375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.2721214531418044,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.4847956403269755,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 21.561941964285715,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 21.78125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001801,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 41.682291666666664,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 42.1015625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.40781182431025204,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.778142757063906,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 28.212611607142858,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 28.47265625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001848,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 45.579427083333336,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 46.265625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle Process RSS (MiB) (Max)"
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
        "date": 1779761829300,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.14820275223423418,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.35049517284335097,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 18.786830357142858,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 18.875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001888,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 39.643229166666664,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 39.89453125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.6099568859610266,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 1.3725889589659737,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 40.736049107142854,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 40.98828125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.002825,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 53.6171875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 54.796875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.11600447325546004,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.28484310519349065,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 17.306919642857142,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 17.4140625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001839,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 37.971354166666664,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 38.125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.08791135300634605,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.2449388867263527,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 15.453683035714286,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 15.55859375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001813,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 38.0078125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 38.11328125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.22131969352648276,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.5413659372323028,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 20.782366071428573,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 21.14453125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.0019,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 41.743489583333336,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 42.14453125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.3143025969629641,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.7931453497429506,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 27.903459821428573,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 28.171875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001814,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 46.075520833333336,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 46.90234375,
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
          "id": "99fda2c577da533a1840085d37ef4d0445c43d3f",
          "message": "Update dependency kubernetes to v36 (#3082)\n\nThis PR contains the following updates:\n\n| Package | Change |\n[Age](https://docs.renovatebot.com/merge-confidence/) |\n[Confidence](https://docs.renovatebot.com/merge-confidence/) |\n|---|---|---|---|\n| [kubernetes](https://redirect.github.com/kubernetes-client/python) |\n`==35.0.0` → `==36.0.0` |\n![age](https://developer.mend.io/api/mc/badges/age/pypi/kubernetes/36.0.0?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/pypi/kubernetes/35.0.0/36.0.0?slim=true)\n|\n\n---\n\n### Release Notes\n\n<details>\n<summary>kubernetes-client/python (kubernetes)</summary>\n\n###\n[`v36.0.0`](https://redirect.github.com/kubernetes-client/python/blob/HEAD/CHANGELOG.md#v3600)\n\n[Compare\nSource](https://redirect.github.com/kubernetes-client/python/compare/v35.0.0...v36.0.0)\n\nKubernetes API Version: v1.36.1\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: (UTC)\n\n- Branch creation\n  - \"before 8am on Monday\"\n- Automerge\n  - At any time (no schedule defined)\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My4xOTQuMCIsInVwZGF0ZWRJblZlciI6IjQzLjE5NC4wIiwidGFyZ2V0QnJhbmNoIjoibWFpbiIsImxhYmVscyI6WyJkZXBlbmRlbmNpZXMiXX0=-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2026-05-26T16:33:01Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/99fda2c577da533a1840085d37ef4d0445c43d3f"
        },
        "date": 1779819757761,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.1542403302479163,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.3703558358638947,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 18.402901785714285,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 18.6171875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001788,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 39.5859375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 39.8359375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.5470739925719469,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 1.3221747935815547,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 40.42299107142857,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 40.72265625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001826,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 54.140625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 55.375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.11403215263681814,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.2902435608123881,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 16.969308035714285,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 17.18359375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001797,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 38.854166666666664,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 39.0234375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.10493153212781824,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.2768987684919836,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 16.115513392857142,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 16.22265625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001783,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 38.134114583333336,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 38.234375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.24663089260983606,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.49338109328764995,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 21.220424107142858,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 21.4609375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001811,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 42.010416666666664,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 42.44140625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.41246624776176843,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.7919395779802227,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 27.91015625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 28.0625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001949,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 46.235677083333336,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 46.9609375,
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
          "id": "0012b9b79d793aa9412b20f86f79094f99a9590d",
          "message": "fix(security): Revert tidy workflow changes and try different approach (#3098)\n\n# Change Summary\n\nReverts #3056\n\nWhile the intention was correct, the change above made CodeQL and\nOpenSSF very unhappy and flagged the `checkout` in `tidy-commit` as a\nDangerous workflow.\n\nInstead, trying to copy what the opentelemetry-collector maintainers did\nin https://github.com/open-telemetry/opentelemetry-collector/pull/15357.\n\nThis is safe because under the `pull_request` trigger ([GitHub\ndocs](https://docs.github.com/en/actions/writing-workflows/choosing-when-your-workflow-runs/events-that-trigger-workflows#pull_request)):\n\n- **Fork PRs cannot access secrets**, and the `GITHUB_TOKEN` is\nread-only — regardless of what `permissions:` the workflow requests. See\n[Workflows in forked\nrepositories](https://docs.github.com/en/actions/writing-workflows/choosing-when-your-workflow-runs/events-that-trigger-workflows#workflows-in-forked-repositories).\n- **Same-repo PRs** get write access, but the job is gated to\n`renovate[bot]` / `dependabot[bot]` actors and explicitly requires\n`github.event.pull_request.head.repo.fork == false`.",
          "timestamp": "2026-05-27T01:05:04Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/0012b9b79d793aa9412b20f86f79094f99a9590d"
        },
        "date": 1779848537960,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.11465120071875912,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.2826064616582328,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 16.732700892857142,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 16.83984375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001766,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 38.65234375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 38.8046875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.7093354659001293,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 1.4342852696288226,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 40.012276785714285,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 40.3828125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001861,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 54.36328125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 55.546875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.0827550158052342,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.2618793304787855,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 16.317522321428573,
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
            "value": 15.001808,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 38.307291666666664,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 38.4140625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.2557818241624388,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.5001416893732971,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 22.40234375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 22.63671875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001814,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 41.966145833333336,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 42.390625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.40158675043787456,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.795331102637927,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 27.506696428571427,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 27.66796875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001854,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 46.020833333333336,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 46.7734375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.15190774617889052,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.3625969173283512,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 17.895089285714285,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 17.99609375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001905,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 39.856770833333336,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 40.109375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle Process RSS (MiB) (Max)"
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
        "date": 1779906865850,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.11885081542242282,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.30370347297928674,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 16.684151785714285,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 16.8359375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001795,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 38.584635416666664,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 38.7265625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.6717953271080146,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 1.2950471000389256,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 41.54408482142857,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 42.109375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.00195,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 54.712239583333336,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 56.3125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.09818471064256212,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.2574752745113309,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 15.721540178571429,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 15.8984375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001806,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 38.03515625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 38.16015625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.24723063496563066,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.4925607476635514,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 21.560267857142858,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 21.953125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001814,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 41.94140625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 42.37890625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.4167750806810962,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.8020309944708357,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 27.515625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 27.76171875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001885,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 46.005208333333336,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 46.76171875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.1496431357414414,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.3408690244850369,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 18.434709821428573,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 18.765625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.00185,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 39.680989583333336,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 39.9375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle Process RSS (MiB) (Max)"
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
        "date": 1779936394367,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.15533860069512787,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.3354730203223546,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 18.228794642857142,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 18.3359375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001853,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 39.846354166666664,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 40.12109375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.6351500697429693,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 1.4013921980845596,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 39.396763392857146,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 39.95703125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001825,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 54.4609375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 56.11328125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.14602627779686891,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.30305295950155764,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 15.740513392857142,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 15.84765625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001811,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 38.572916666666664,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 38.75,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.1083110956680367,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.24817545496966867,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 16.036830357142858,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 16.140625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001971,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 37.645833333333336,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 37.7734375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.2464280930763133,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.4618402615600187,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 21.625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 21.97265625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001754,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 41.763020833333336,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 42.21875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.3250662470666698,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.7412519464341327,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 27.808035714285715,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 27.95703125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001776,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 46.348958333333336,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 47.09765625,
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
          "id": "70292bb27def0ed7a179f4cf46ff305dcebd096a",
          "message": "Update one_collect digest to 293b7d3 (#3114)\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n|---|---|---|---|\n| one_collect | workspace.dependencies | digest | `6ccba44` → `293b7d3`\n|\n\n---\n\n### Configuration\n\n📅 **Schedule**: (UTC)\n\n- Branch creation\n  - At any time (no schedule defined)\n- Automerge\n  - At any time (no schedule defined)\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My4xOTQuMCIsInVwZGF0ZWRJblZlciI6IjQzLjE5OC4wIiwidGFyZ2V0QnJhbmNoIjoibWFpbiIsImxhYmVscyI6WyJkZXBlbmRlbmNpZXMiXX0=-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-05-28T17:05:14Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/70292bb27def0ed7a179f4cf46ff305dcebd096a"
        },
        "date": 1779992838504,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.11754745156051585,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.2874567824326429,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 17.064174107142858,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 17.44921875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.00181,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 38.1171875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 38.265625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.645299244556856,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 1.4295625097306555,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 39.48325892857143,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 39.9921875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001895,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 54.020833333333336,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 55.23046875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.06234385417356894,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.2586100085609775,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 15.301897321428571,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 15.41015625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001785,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 37.919270833333336,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 38.02734375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.2689640577013541,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.48261641488864665,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 21.40234375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 21.625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001829,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 41.486979166666664,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 41.921875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.3190522916327415,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.7524546516154146,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 27.582589285714285,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 27.84375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001965,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 46.30078125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 47.125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.14655373069545613,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.3538665212989643,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 17.860491071428573,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 17.8671875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001895,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 39.47265625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 39.7265625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle Process RSS (MiB) (Max)"
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
        "date": 1780021400613,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.11128115067060615,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.26983029736883074,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 16.365513392857142,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 16.47265625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001834,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 38.651041666666664,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 38.84375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.656761880652956,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 1.3563435334423422,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 40.335379464285715,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 40.796875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001815,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 55.3828125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 56.98828125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.10109779602218028,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.2713883049131823,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 15.96875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 16.25,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001789,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 37.66796875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 37.78515625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.21035295061881393,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.5140722291407223,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 21.493861607142858,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 21.78515625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001765,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 42.295572916666664,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 42.71875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.38305782829913704,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.7662659607598878,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 28.714285714285715,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 28.90234375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001789,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 46.86328125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 47.56640625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.15089362484806426,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.3592213050926647,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 18.19140625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 18.4140625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.00186,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 39.326822916666664,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 39.6015625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle Process RSS (MiB) (Max)"
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
        "date": 1780079738376,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.11977013339981837,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.31048419741553795,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 16.243861607142858,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 16.63671875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001874,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 38.479166666666664,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 38.6640625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.6198162037450174,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 1.3838107098381072,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 39.929129464285715,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 40.32421875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001807,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 54.432291666666664,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 56.0390625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.10609091156727943,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.2647725856697819,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 16.516183035714285,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 16.51953125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001807,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 38.678385416666664,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 38.82421875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.2549211064534364,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.4841734797165771,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 21.713169642857142,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 22.0546875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001812,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 42.408854166666664,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 42.87890625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.3927089848274809,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.7677209374756677,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 28.326450892857142,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 28.55859375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001829,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 45.966145833333336,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 46.859375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.14662266403501548,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.3502413202553324,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 18.047991071428573,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 18.125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001754,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 39.713541666666664,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 39.98828125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle Process RSS (MiB) (Max)"
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
        "date": 1780109728960,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.14073887677213484,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.28993539347707636,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 15.858258928571429,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 15.8671875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001866,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 39.19921875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 39.375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.6751828658946473,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 1.3691567013511414,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 40.51841517857143,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 40.73828125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.003498,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 55.350260416666664,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 56.5546875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.10464951257833646,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.27607256871447483,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 16.015066964285715,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 16.1875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001817,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 38.578125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 38.68359375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.26419597438772635,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.515406162464986,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 21.462611607142858,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 21.64453125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001913,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 41.934895833333336,
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
            "value": 0.33325010122435816,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.8055790784557908,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 28.558035714285715,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 28.8203125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001817,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 46.70703125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 47.65234375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.17046032800741556,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.3839700957869325,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 17.825892857142858,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 18.01953125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001794,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 39.9453125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 40.20703125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle Process RSS (MiB) (Max)"
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
        "date": 1780163273841,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.11622110947526472,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.29448769853628154,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 16.523995535714285,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 16.62890625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.00181,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 38.627604166666664,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 38.796875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.6166548092910872,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 1.3525732087227416,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 39.28125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 39.4609375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001852,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 54.485677083333336,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 55.69140625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.1186633653198835,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.2488335925349922,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 15.601004464285714,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 15.61328125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001763,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 38.276041666666664,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 38.3828125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.25582533995050183,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.507639418215758,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 21.994977678571427,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 22.10546875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001884,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 42.135416666666664,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 42.56640625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.3487797115175003,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.7449556143902819,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 28.263392857142858,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 28.64453125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001785,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 46.638020833333336,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 47.375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.1685121028404488,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.3694855630788388,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 17.98046875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 18.08984375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001843,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 39.455729166666664,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 39.7109375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle Process RSS (MiB) (Max)"
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
        "date": 1780195015607,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.13931481333407914,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.2978829389788294,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 17.196428571428573,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 17.375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.00187,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 39.15625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 39.30859375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.6872899294495771,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 1.3594207862981706,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 40.93638392857143,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 41.34765625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.00182,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 55.151041666666664,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 56.703125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.10905245568667075,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.25390735694822886,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 16.090959821428573,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 16.09765625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001796,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 38.307291666666664,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 38.4140625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.27373746805281285,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.4840361258175023,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 21.84765625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 22.546875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001849,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 42.217447916666664,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 42.640625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.38514156940793415,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.8074612689762553,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 28.23828125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 28.50390625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.00186,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 46.252604166666664,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 47.01953125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.15405455848607946,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.3917228493577268,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 18.309151785714285,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 18.4140625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001832,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 39.71875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 39.97265625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle Process RSS (MiB) (Max)"
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
        "date": 1780249953630,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.11581417029864918,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.28694636880205493,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 16.65234375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 16.87890625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001846,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 38.259114583333336,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 38.43359375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.6837128829061625,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 1.319288211982283,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 39.77455357142857,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 39.8515625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001949,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 54.79296875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 55.9921875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.11783258886930614,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.26695719844357974,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 15.653459821428571,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 16.11328125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001797,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 38.563802083333336,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 38.68359375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.27682074368761345,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.493073155562466,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 20.567522321428573,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 21.02734375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001874,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 41.9921875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 42.421875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.4205033784056351,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.788926430517711,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 28.397879464285715,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 28.640625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001868,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 46.436197916666664,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 47.41015625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.1471632755977439,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.35784196185286105,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 18.758370535714285,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 18.8671875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001808,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 39.657552083333336,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 39.9296875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle Process RSS (MiB) (Max)"
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
        "date": 1780281567945,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.11929336387956829,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.2835140186915888,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 16.38671875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 16.38671875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.00184,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 38.451822916666664,
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
            "value": 0.6824768839296542,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 1.3552355001946281,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 40.13169642857143,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 40.72265625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001917,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 55.462239583333336,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 57.05859375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.1130751646499881,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.2703706010588602,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 15.928013392857142,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 16.03515625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.00177,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 37.8828125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 37.984375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.2077216239093616,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.477370921267814,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 21.981026785714285,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 22.0859375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001827,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 41.696614583333336,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 42.1484375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.3751951335677589,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.817617193583554,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 28.401785714285715,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 28.8125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001869,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 46.66796875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 47.62890625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.1480317874321836,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.3623826208829713,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 18.004464285714285,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 18.25390625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001781,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 39.72265625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 39.9765625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle Process RSS (MiB) (Max)"
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
        "date": 1780342141221,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.13175075589106022,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.2991250194613109,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 16.637834821428573,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 16.71875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001777,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 38.930989583333336,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 39.10546875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.6452849540778626,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 1.420373599003736,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 40.482142857142854,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 40.98046875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001927,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 55.313802083333336,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 56.90234375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.1117505559169029,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.25873082613096626,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 15.709821428571429,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 15.83984375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001842,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 38.216145833333336,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 38.34375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.26340435592920847,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.47157894736842104,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 20.970424107142858,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 21.17578125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001825,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 42.28125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 42.7109375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.3309403468593018,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.7583339820942001,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 28.359375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 28.703125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001902,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 46.584635416666664,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 47.3125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.1563681822912268,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.3814101167315175,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 17.698102678571427,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 17.8515625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001729,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 39.811197916666664,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 40.0859375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle Process RSS (MiB) (Max)"
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
        "date": 1780370315917,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.13238902533188945,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.2953309453356175,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 16.337053571428573,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 16.51171875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001846,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 38.983072916666664,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 39.15234375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.6536371510901247,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 1.3868991282689913,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 39.695870535714285,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 40.06640625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001871,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 54.981770833333336,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 56.1796875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.10133373158285805,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.2762288828337875,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 15.760044642857142,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 15.91796875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001899,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 38.263020833333336,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 38.390625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.2959512628001082,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.6563568773234201,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 21.130580357142858,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 21.60546875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001785,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 42.033854166666664,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 42.484375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.38966233028273445,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.7603516688710806,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 27.954799107142858,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 28.1328125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001843,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 46.662760416666664,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 47.390625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.16719945306187756,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.35349155707727026,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 17.825334821428573,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 18.046875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001805,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 39.92578125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 40.19921875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle Process RSS (MiB) (Max)"
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
        "date": 1780427535469,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.11236599686288917,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.2787980694379574,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 16.446986607142858,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 16.609375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001847,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 38.752604166666664,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 38.9296875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.6734112088208325,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 1.352855251888188,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 41.219308035714285,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 41.91796875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001865,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 55.06640625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 56.66796875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.10250909189669416,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.2677540856031128,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 15.578683035714286,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 15.859375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001864,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 38.608072916666664,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 38.73046875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.2709706558036735,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.5034480610496808,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 20.21875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 20.6015625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.00189,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 42.4453125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 42.89453125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.3949088331346633,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.7650689306020718,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 27.958705357142858,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 28.23828125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.00182,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 46.891927083333336,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 47.72265625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.1486490905879935,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.36211182058869334,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 17.820870535714285,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 17.84765625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001751,
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
        "date": 1780455976691,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.1185419208640156,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.2956681453809635,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 16.385044642857142,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 16.4921875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001828,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 39.21875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 39.39453125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.614539705979626,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 1.4569640354974311,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 39.81752232142857,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 40.2265625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001875,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 55.278645833333336,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 56.49609375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.1040203838992919,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.28340365901128844,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 16.459821428571427,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 16.4609375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001784,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 38.66015625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 38.80859375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.24887842138786348,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.4833517638813177,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 21.4375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 21.6640625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001899,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 42.403645833333336,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 42.87109375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.3841149189668142,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.7726251849544428,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 28.233816964285715,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 28.38671875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.00186,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 46.299479166666664,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 47.1953125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.17262427117985737,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.37962165083777316,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 17.479910714285715,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 17.703125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001871,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 39.727864583333336,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 40.0234375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle Process RSS (MiB) (Max)"
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
        "date": 1780514070536,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.12004265468684855,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.3011881959043837,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 16.418526785714285,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 16.6328125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001875,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 39.16015625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 39.3359375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.6592219178290492,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 1.3818340339405262,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 39.983816964285715,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 40.57421875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001925,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 54.505208333333336,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 56.125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.08778061297806934,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.26547040498442365,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 16.308035714285715,
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
            "value": 15.001809,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 38.734375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 38.859375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.27443681821227445,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.5022343324250681,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 21.620535714285715,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 21.87109375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001841,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 41.614583333333336,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 42.0859375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.39818959040684765,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.7577767398411958,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 28.153459821428573,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 28.39453125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001872,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 46.157552083333336,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 47.0546875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.1675240199329088,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.3462286915233128,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 18.618861607142858,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 18.7265625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001904,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 40.158854166666664,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 40.43359375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle Process RSS (MiB) (Max)"
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
        "date": 1780543776466,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.12816892663024188,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.312800311405216,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 16.608816964285715,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 16.796875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001779,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 39.252604166666664,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 39.41796875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.6427194268728226,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 1.4440336134453782,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 39.180245535714285,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 39.6171875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001939,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 55.005208333333336,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 56.203125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.10685529946635301,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.29092268161644474,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 15.608816964285714,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 15.890625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001756,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 38.891927083333336,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 39.01171875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.21444174270962407,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.5172432053578382,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 21.331473214285715,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 21.6171875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001797,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 42.34375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 42.7890625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.3305706819915187,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.7172585669781931,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 28.128348214285715,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 28.4921875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001873,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 46.973958333333336,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 47.74609375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.10669600967860274,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.38966477405304506,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 18.450334821428573,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 18.5546875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001795,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 40.311197916666664,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 40.5859375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle Process RSS (MiB) (Max)"
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
        "date": 1780600544043,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.11603389415567925,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.2768133322949926,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 16.072544642857142,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 16.125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001875,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 38.453125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 38.60546875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.6925338066919733,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 1.3371989100817439,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 39.357700892857146,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 40.04296875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001824,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 54.602864583333336,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 56.18359375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.10829786973842173,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.31925268566090614,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 16.731026785714285,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 17.08984375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.007186,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 38.483072916666664,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 38.62890625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.25851290826262313,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.524297726564933,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 21.399553571428573,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 21.9453125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001869,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 42.119791666666664,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 42.59375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.33837310338228854,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.7570160411150911,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 28.91796875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 29.26953125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001881,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 46.21484375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 47.15234375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.1625900314313695,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.3783810709838107,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 18.531808035714285,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 18.64453125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001875,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 40.21484375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 40.51171875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle Process RSS (MiB) (Max)"
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
        "date": 1780628316054,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.07611580207041307,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.2883762946810996,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 16.802455357142858,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 16.98046875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001889,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 38.96484375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 39.140625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 2 Cores/Idle State Baseline - 2 Cores - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.6854127782738406,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 1.410482049684604,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 40.175223214285715,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 40.3984375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001875,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 55.1796875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 56.3828125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 32 Cores/Idle State Baseline - 32 Cores - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.09432817628749855,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.2537089494163424,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 15.563058035714286,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 15.77734375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001761,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 38.385416666666664,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 38.515625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.26879906748235227,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.49621057550035047,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 20.59375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 21.046875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001835,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 42.359375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 42.8203125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 8 Cores/Idle State Baseline - 8 Cores - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.3242853996179451,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.7785453979130976,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 28.058035714285715,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 28.37109375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001855,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 46.84375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 47.796875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 16 Cores/Idle State Baseline - 16 Cores - Idle Process RSS (MiB) (Max)"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.1858068107173068,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.381049680735088,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 18.436941964285715,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 18.453125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001855,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle Test Duration"
          },
          {
            "name": "idle_rss_mib_avg",
            "value": 40.046875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle Process RSS (MiB) (Avg)"
          },
          {
            "name": "idle_rss_mib_max",
            "value": 40.33984375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - 4 Cores/Idle State Baseline - 4 Cores - Idle Process RSS (MiB) (Max)"
          }
        ]
      }
    ]
  }
}