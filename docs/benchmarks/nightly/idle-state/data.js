window.BENCHMARK_DATA = {
  "lastUpdate": 1779216281404,
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
      }
    ]
  }
}