window.BENCHMARK_DATA = {
  "lastUpdate": 1780079739112,
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
      }
    ]
  }
}